//! Flatten `vueCompilerOptions` the way `@vue/language-core` does: later
//! `extends` entries override earlier ones, then the extending file wins.
//!
//! `checkUnknownProps` defaults to `strictTemplates`, which defaults to false.
//! Isolated typecheck must follow that, or extras vue-tsc would ignore (vuestic
//! `:gradient`, voicevox `disable`) score as one-sided false positives.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashMap, FxHashSet};

use crate::batch::error::CorsaResult;
use crate::virtual_ts::ResolveStyleClassNames;

use super::super::VirtualProject;
use super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path,
};

/// Read the Vue dialect from an effective TypeScript project's standard
/// `vueCompilerOptions.target`, including JSONC and standard `extends` chains.
/// The caller selects the owning reference project and gives an explicitly
/// configured Vite Vue dialect precedence over this value.
///
/// Vue minor targets select the corresponding supported language family;
/// `99` is vue-tsc's latest-Vue target. Invalid or unsupported explicit values
/// are reported instead of silently selecting Vue 3.
pub fn snapshot_tsconfig_vue_target(
    project_root: &Path,
    tsconfig_path: &Path,
) -> CorsaResult<Option<vize_carton::config::VueVersion>> {
    let options = load_vue_compiler_options(Some(tsconfig_path))?;
    let Some(target) = options.get("target") else {
        return Ok(None);
    };
    if target.as_str() == Some("auto") {
        let directory = tsconfig_path.parent().unwrap_or(project_root);
        let (manifest_path, manifest) = crate::package_route::installed_package_manifest(
            directory, "vue",
        )
        .ok_or_else(|| {
            crate::batch::error::CorsaError::Configuration(vize_carton::cstr!(
                "cannot resolve installed Vue for vueCompilerOptions.target \"auto\" in {}",
                tsconfig_path.display()
            ))
        })?;
        let version = manifest
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let mut parts = version.split('.');
        let major = parts.next().and_then(|part| part.parse::<u32>().ok());
        let minor = parts.next().and_then(|part| part.parse::<u32>().ok());
        use vize_carton::config::VueVersion;
        return match (major, minor) {
            (Some(2), Some(0..=6)) => Ok(Some(VueVersion::V2)),
            (Some(2), Some(7)) => Ok(Some(VueVersion::V2_7)),
            (Some(3), Some(_)) => Ok(Some(VueVersion::V3)),
            _ => Err(crate::batch::error::CorsaError::Configuration(
                vize_carton::cstr!(
                    "unsupported installed Vue version {version:?} in {} for vueCompilerOptions.target \"auto\"",
                    manifest_path.display()
                ),
            )),
        };
    }
    let version = target.as_f64().and_then(|target| {
        use vize_carton::config::VueVersion;
        if (2.0..2.7).contains(&target) {
            Some(VueVersion::V2)
        } else if target == 2.7 {
            Some(VueVersion::V2_7)
        } else if (3.0..4.0).contains(&target) || target == 99.0 {
            Some(VueVersion::V3)
        } else {
            None
        }
    });
    version.map(Some).ok_or_else(|| {
        crate::batch::error::CorsaError::Configuration(vize_carton::cstr!(
            "unsupported vueCompilerOptions.target {target} in {}; expected a numeric Vue 2.x target up to 2.7, a Vue 3.x target, 99, or \"auto\"",
            tsconfig_path.display()
        ))
    })
}

#[derive(Default)]
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
struct ChainLoad {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Map<std::string::String, Value>>,
    cycle_cut: bool,
}

impl VirtualProject {
    pub(in super::super) fn vue_target_manifest_path(&self, config: &Path) -> Option<PathBuf> {
        let options = load_vue_compiler_options(Some(config)).ok()?;
        if options.get("target").and_then(Value::as_str) != Some("auto") {
            return None;
        }
        crate::package_route::installed_package_manifest(
            config.parent().unwrap_or(&self.project_root),
            "vue",
        )
        .map(|(path, _)| path)
    }

    pub(in super::super) fn refresh_vue_compiler_options(&mut self) {
        let path = self.resolved_tsconfig_path();
        let options = load_vue_compiler_options(path.as_deref()).ok();
        self.typed_router.root =
            super::super::typed_router::configured_root(self, options.as_ref(), path.as_deref());
        self.virtual_ts_check_options.check_unknown_props = if path.is_none() {
            true
        } else {
            options
                .as_ref()
                .map(check_unknown_props_enabled)
                .unwrap_or(true)
        };
        self.virtual_ts_check_options
            .check_unknown_fallthrough_props = options.as_ref().is_some_and(|options| {
            options.get("checkUnknownProps").and_then(Value::as_bool) == Some(true)
        });
        self.virtual_ts_check_options.strict_css_modules = options
            .as_ref()
            .and_then(|options| options.get("strictCssModules").and_then(Value::as_bool))
            .unwrap_or(false);
        self.virtual_ts_check_options.check_unknown_components = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("checkUnknownComponents")
                    .or_else(|| options.get("strictTemplates"))
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.check_unknown_events = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("checkUnknownEvents")
                    .or_else(|| options.get("strictTemplates"))
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.check_unknown_directives = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("checkUnknownDirectives")
                    .or_else(|| options.get("strictTemplates"))
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.strict_v_model = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("strictVModel")
                    .or_else(|| options.get("strictTemplates"))
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.vapor = options
            .as_ref()
            .and_then(|options| options.get("vapor").and_then(Value::as_bool))
            .unwrap_or(false);
        self.virtual_ts_check_options.infer_component_dollar_el = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("inferComponentDollarEl")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.infer_template_dollar_el = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("inferTemplateDollarEl")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        let flag = |name: &str| {
            options
                .as_ref()
                .and_then(|options| options.get(name).and_then(Value::as_bool))
                .unwrap_or(false)
        };
        self.virtual_ts_check_options.strict_component_attrs = flag("strictComponentAttrs");
        self.virtual_ts_check_options.infer_component_dollar_refs =
            flag("inferComponentDollarRefs");
        self.virtual_ts_check_options.infer_template_dollar_refs = flag("inferTemplateDollarRefs");
        self.virtual_ts_check_options.jsx_slots = options
            .as_ref()
            .and_then(|options| options.get("jsxSlots").and_then(Value::as_bool))
            .unwrap_or(false);
        self.virtual_ts_check_options.infer_template_dollar_slots = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("inferTemplateDollarSlots")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.infer_template_dollar_attrs = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("inferTemplateDollarAttrs")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.fallthrough_attributes = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("fallthroughAttributes")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options
            .check_required_fallthrough_attributes = options
            .as_ref()
            .and_then(|options| {
                options
                    .get("checkRequiredFallthroughAttributes")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false);
        self.virtual_ts_check_options.resolve_style_class_names = match options
            .as_ref()
            .and_then(|options| options.get("resolveStyleClassNames"))
        {
            Some(Value::Bool(true)) => ResolveStyleClassNames::All,
            Some(Value::Bool(false)) => ResolveStyleClassNames::None,
            _ => ResolveStyleClassNames::Scoped,
        };
        self.virtual_ts_check_options.resolve_style_imports = options
            .as_ref()
            .and_then(|options| options.get("resolveStyleImports").and_then(Value::as_bool))
            .unwrap_or(false);
    }

    #[cfg(test)]
    pub(crate) fn checks_unknown_props(&self) -> bool {
        self.virtual_ts_check_options.check_unknown_props
    }
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn load_vue_compiler_options(
    tsconfig_path: Option<&Path>,
) -> CorsaResult<Map<std::string::String, Value>> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Ok(Map::new());
    };
    let mut load = ChainLoad::default();
    load_vue_compiler_options_inner(tsconfig_path, &mut load)
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn load_vue_compiler_options_inner(
    tsconfig_path: &Path,
    load: &mut ChainLoad,
) -> CorsaResult<Map<std::string::String, Value>> {
    if !tsconfig_path.exists() {
        return Ok(Map::new());
    }
    let normalized = normalize_path_lexically(tsconfig_path);
    if let Some(cached) = load.completed.get(&normalized) {
        return Ok(cached.clone());
    }
    if !load.active.insert(normalized.clone()) {
        load.cycle_cut = true;
        return Ok(Map::new());
    }
    let enclosing_cycle_cut = std::mem::replace(&mut load.cycle_cut, false);
    let flattened = merge_extended_vue_compiler_options(&normalized, load);
    load.active.remove(&normalized);
    let chain_cycle_cut = load.cycle_cut;
    load.cycle_cut = enclosing_cycle_cut || chain_cycle_cut;
    if !chain_cycle_cut && let Ok(flattened) = &flattened {
        load.completed.insert(normalized, flattened.clone());
    }
    flattened
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn merge_extended_vue_compiler_options(
    normalized: &Path,
    load: &mut ChainLoad,
) -> CorsaResult<Map<std::string::String, Value>> {
    let content = std::fs::read_to_string(normalized)?;
    let config = parse_jsonc_value(&content)?;
    let current = config
        .get("vueCompilerOptions")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut inherited = Map::new();
    match config.get("extends") {
        Some(Value::String(extends)) => {
            if let Some(parent_path) = resolve_extended_tsconfig_path(normalized, extends) {
                inherited = load_vue_compiler_options_inner(&parent_path, load)?;
            }
        }
        Some(Value::Array(entries)) => {
            for extends in entries.iter().filter_map(Value::as_str) {
                if let Some(parent_path) = resolve_extended_tsconfig_path(normalized, extends) {
                    inherited.extend(load_vue_compiler_options_inner(&parent_path, load)?);
                }
            }
        }
        _ => {}
    }
    inherited.extend(current);
    Ok(inherited)
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn check_unknown_props_enabled(options: &Map<std::string::String, Value>) -> bool {
    if let Some(value) = options.get("checkUnknownProps").and_then(Value::as_bool) {
        return value;
    }
    options
        .get("strictTemplates")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "vue_compiler_options_tests.rs"]
mod tests;
