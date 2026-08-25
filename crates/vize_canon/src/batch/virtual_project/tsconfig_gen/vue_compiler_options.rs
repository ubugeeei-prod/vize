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
use super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path,
};

#[derive(Default)]
#[allow(clippy::disallowed_types)]
struct ChainLoad {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Map<std::string::String, Value>>,
    cycle_cut: bool,
}

impl VirtualProject {
    pub(in super::super) fn resolve_check_unknown_props(&self) -> bool {
        let Some(tsconfig_path) = self.resolved_tsconfig_path() else {
            return true;
        };
        self.load_vue_compiler_options(Some(tsconfig_path.as_path()))
            .ok()
            .as_ref()
            .map(check_unknown_props_enabled)
            .unwrap_or(true)
    }

    #[cfg(test)]
    pub(crate) fn checks_unknown_props(&self) -> bool {
        self.virtual_ts_check_options.check_unknown_props
    }

    #[allow(clippy::disallowed_types)]
    fn load_vue_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };
        let mut load = ChainLoad::default();
        self.load_vue_compiler_options_inner(tsconfig_path, &mut load)
    }

    #[allow(clippy::disallowed_types)]
    fn load_vue_compiler_options_inner(
        &self,
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
        let flattened = self.merge_extended_vue_compiler_options(&normalized, load);
        load.active.remove(&normalized);
        let chain_cycle_cut = load.cycle_cut;
        load.cycle_cut = enclosing_cycle_cut || chain_cycle_cut;
        if !chain_cycle_cut && let Ok(flattened) = &flattened {
            load.completed.insert(normalized, flattened.clone());
        }
        flattened
    }

    #[allow(clippy::disallowed_types)]
    fn merge_extended_vue_compiler_options(
        &self,
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
                    inherited = self.load_vue_compiler_options_inner(&parent_path, load)?;
                }
            }
            Some(Value::Array(entries)) => {
                for extends in entries.iter().filter_map(Value::as_str) {
                    if let Some(parent_path) = resolve_extended_tsconfig_path(normalized, extends) {
                        inherited.extend(self.load_vue_compiler_options_inner(&parent_path, load)?);
                    }
                }
            }
            _ => {}
        }
        inherited.extend(current);
        Ok(inherited)
    }
}

#[allow(clippy::disallowed_types)]
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
mod tests {
    use super::super::super::VirtualProject;
    use super::check_unknown_props_enabled;
    use serde_json::{Map, Value};
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(cstr!("vue-compiler-options-{name}-{}", std::process::id()).as_str())
    }

    #[allow(clippy::disallowed_types)]
    fn bool_map(pairs: &[(&str, bool)]) -> Map<std::string::String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), Value::Bool(*value)))
            .collect()
    }

    #[test]
    fn unknown_props_follow_vue_tsc_defaults() {
        assert!(!check_unknown_props_enabled(&Map::new()));
        assert!(check_unknown_props_enabled(&bool_map(&[(
            "strictTemplates",
            true
        )])));
        assert!(!check_unknown_props_enabled(&bool_map(&[
            ("strictTemplates", true),
            ("checkUnknownProps", false),
        ])));
        assert!(check_unknown_props_enabled(&bool_map(&[(
            "checkUnknownProps",
            true
        )])));
    }

    #[test]
    fn tsconfig_without_vue_compiler_options_matches_vue_tsc() {
        let case_dir = unique_case_dir("default");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(&case_dir).unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{ "compilerOptions": { "strict": true } }"#,
        )
        .unwrap();
        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
        assert!(!project.checks_unknown_props());
        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn extended_strict_templates_enables_unknown_props() {
        let case_dir = unique_case_dir("extends");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(&case_dir).unwrap();
        fs::write(
            case_dir.join("tsconfig.base.json"),
            r#"{ "vueCompilerOptions": { "strictTemplates": true } }"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{ "extends": "./tsconfig.base.json", "compilerOptions": { "strict": true } }"#,
        )
        .unwrap();
        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
        assert!(project.checks_unknown_props());
        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn no_tsconfig_keeps_generated_virtual_ts_strict() {
        let case_dir = unique_case_dir("none");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(&case_dir).unwrap();
        let project = VirtualProject::new(&case_dir).unwrap();
        assert!(project.checks_unknown_props());
        let _ = fs::remove_dir_all(&case_dir);
    }
}
