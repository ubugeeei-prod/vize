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
use super::compiler_inputs::CompilerInputs;

use extends::check_unknown_props_enabled;
pub(in super::super) use extends::snapshot_vue_compiler_options;
pub(super) use extends::{load_vue_compiler_options, load_vue_compiler_options_with_inputs};

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
    vue_target_from_options(project_root, tsconfig_path, &options, None)
        .map(|(version, _manifest)| version)
}

/// Flatten the effective `vueCompilerOptions` for a TypeScript project.
///
/// Generated checker configs use this alongside the compiler-options snapshot
/// so selecting the generated config does not discard Vue-specific semantics.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub fn snapshot_tsconfig_vue_compiler_options(
    tsconfig_path: &Path,
) -> CorsaResult<Map<std::string::String, Value>> {
    snapshot_vue_compiler_options(tsconfig_path).map(|(options, _)| options)
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in crate::batch::virtual_project) fn vue_target_from_options(
    project_root: &Path,
    tsconfig_path: &Path,
    options: &Map<std::string::String, Value>,
    inputs: Option<&CompilerInputs>,
) -> CorsaResult<(
    Option<vize_carton::config::VueVersion>,
    Option<std::path::PathBuf>,
)> {
    vue_target_from_options_with_resolver(project_root, tsconfig_path, options, |directory| {
        inputs.map_or_else(
            || crate::package_route::installed_package_manifest(directory, "vue"),
            |inputs| inputs.installed_vue_manifest(directory),
        )
    })
}

/// Preserve the existing package resolver's actual probes for dynamic hosts.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in crate::batch::virtual_project) fn vue_target_from_options_with_dependencies(
    project_root: &Path,
    tsconfig_path: &Path,
    options: &Map<std::string::String, Value>,
    dependencies: &mut Vec<PathBuf>,
) -> CorsaResult<(
    Option<vize_carton::config::VueVersion>,
    Option<std::path::PathBuf>,
)> {
    vue_target_from_options_with_resolver(project_root, tsconfig_path, options, |directory| {
        let dependencies = std::cell::RefCell::new(dependencies);
        let mut manifests = FxHashMap::default();
        crate::package_route::installed_package_manifest_with_reader(
            directory,
            "vue",
            |path| {
                manifests
                    .entry(path.to_path_buf())
                    .or_insert_with(|| {
                        let source = std::fs::read_to_string(path).ok()?;
                        serde_json::from_str(&source).ok()
                    })
                    .clone()
            },
            |path| dependencies.borrow_mut().push(path.to_path_buf()),
        )
    })
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn vue_target_from_options_with_resolver(
    project_root: &Path,
    tsconfig_path: &Path,
    options: &Map<std::string::String, Value>,
    resolve_manifest: impl FnOnce(&Path) -> Option<(PathBuf, Value)>,
) -> CorsaResult<(
    Option<vize_carton::config::VueVersion>,
    Option<std::path::PathBuf>,
)> {
    let Some(target) = options.get("target") else {
        return Ok((None, None));
    };
    if target.as_str() == Some("auto") {
        let directory = tsconfig_path.parent().unwrap_or(project_root);
        let (manifest_path, manifest) = resolve_manifest(directory).ok_or_else(|| {
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
            (Some(2), Some(0..=6)) => Ok((Some(VueVersion::V2), Some(manifest_path))),
            (Some(2), Some(7)) => Ok((Some(VueVersion::V2_7), Some(manifest_path))),
            (Some(3), Some(_)) => Ok((Some(VueVersion::V3), Some(manifest_path))),
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
    version
        .map(|version| (Some(version), None))
        .ok_or_else(|| {
        crate::batch::error::CorsaError::Configuration(vize_carton::cstr!(
            "unsupported vueCompilerOptions.target {target} in {}; expected a numeric Vue 2.x target up to 2.7, a Vue 3.x target, 99, or \"auto\"",
            tsconfig_path.display()
        ))
    })
}

#[derive(Default)]
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
struct ChainLoad<'a> {
    inputs: Option<&'a CompilerInputs>,
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Map<std::string::String, Value>>,
    cycle_cut: bool,
    dependencies: Option<Vec<PathBuf>>,
}

impl VirtualProject {
    pub(in super::super) fn vue_target_manifest_path_with_inputs(
        &self,
        config: &Path,
        inputs: Option<&CompilerInputs>,
    ) -> Option<PathBuf> {
        let options = load_vue_compiler_options_with_inputs(Some(config), inputs).ok()?;
        if options.get("target").and_then(Value::as_str) != Some("auto") {
            return None;
        }
        let directory = config.parent().unwrap_or(&self.project_root);
        inputs
            .map_or_else(
                || crate::package_route::installed_package_manifest(directory, "vue"),
                |inputs| inputs.installed_vue_manifest(directory),
            )
            .map(|(path, _)| path)
    }

    pub(in super::super) fn refresh_vue_compiler_options(&mut self) {
        let path = self.resolved_tsconfig_path();
        let options = self.compiler_configuration.as_ref().map_or_else(
            || load_vue_compiler_options(path.as_deref()).ok(),
            |snapshot| Some(snapshot.vue_options.clone()),
        );
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
        let fallthrough_attributes = options
            .as_ref()
            .and_then(|options| options.get("fallthroughAttributes"));
        self.virtual_ts_check_options
            .fallthrough_attributes_specified = fallthrough_attributes.is_some();
        self.virtual_ts_check_options.fallthrough_attributes = fallthrough_attributes
            .and_then(Value::as_bool)
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
        self.virtual_ts_check_options.vue_compiler_target =
            self.resolved_vue_compiler_target(path.as_deref(), options.as_ref());
    }

    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    fn resolved_vue_compiler_target(
        &self,
        path: Option<&Path>,
        options: Option<&Map<std::string::String, Value>>,
    ) -> Option<vize_carton::config::VueVersion> {
        if let Some(snapshot) = self.compiler_configuration.as_ref() {
            return snapshot.vue_target().ok().flatten();
        }
        let (Some(path), Some(options)) = (path, options) else {
            return None;
        };
        vue_target_from_options(&self.project_root, path, options, None)
            .ok()
            .and_then(|(version, _manifest)| version)
    }

    #[cfg(test)]
    pub(crate) fn checks_unknown_props(&self) -> bool {
        self.virtual_ts_check_options.check_unknown_props
    }
}

mod extends;

#[cfg(test)]
mod tests;
