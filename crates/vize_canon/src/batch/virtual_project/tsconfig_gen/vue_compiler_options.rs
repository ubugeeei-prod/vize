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

use super::super::VirtualProject;
use super::compiler_inputs::CompilerInputs;

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

/// Whether an unimported `useTemplateRef(...)` call is Vue's composable.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::batch::virtual_project) struct GlobalTemplateRef {
    /// `vueCompilerOptions.composables.useTemplateRef`, when the project sets it.
    pub(in crate::batch::virtual_project) explicit: Option<bool>,
    /// The program declares the global as Vue's own export (Nuxt's auto-imports).
    pub(in crate::batch::virtual_project) ambient: bool,
}

impl GlobalTemplateRef {
    /// An explicit option decides; otherwise only Vue's own export counts.
    pub(in crate::batch::virtual_project) fn enabled(self) -> bool {
        self.explicit.unwrap_or(self.ambient)
    }
}

impl super::super::VirtualProject {
    /// The program declares the global `useTemplateRef` as Vue's own export
    /// (Nuxt's auto-import declarations do). Absent an explicit
    /// `composables.useTemplateRef`, the call then reads the template's refs.
    pub(crate) fn set_ambient_vue_template_ref(&mut self, declared: bool) {
        self.global_template_ref.ambient = declared;
    }
}

/// What `vueCompilerOptions.composables.useTemplateRef` says about an
/// unimported `useTemplateRef(...)` call: `Some(true)` when the list names it,
/// `Some(false)` when the project lists other names (or none), and `None` when
/// the option is absent. Vue's language tools then recognize the call by name;
/// Vize does so only when the global is declared as Vue's own export (see
/// [`crate::batch::declares_vue_template_ref_global`]), so a separately
/// authored ambient composable keeps its contract (#10980).
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in crate::batch::virtual_project) fn configured_global_template_ref(
    options: &Map<std::string::String, Value>,
) -> Option<bool> {
    let names = options
        .get("composables")
        .and_then(Value::as_object)
        .and_then(|composables| composables.get("useTemplateRef"))?;
    Some(names.as_array().is_some_and(|names| {
        names
            .iter()
            .any(|name| name.as_str() == Some("useTemplateRef"))
    }))
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
        self.global_template_ref.explicit =
            options.as_ref().and_then(configured_global_template_ref);
        super::super::typed_router::refresh(self, options.as_ref(), path.as_deref());
        super::super::vue_check_options::resolve(
            &mut self.virtual_ts_check_options,
            options.as_ref(),
        );
        if path.is_none() {
            self.virtual_ts_check_options.check_unknown_props = true;
        }
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
