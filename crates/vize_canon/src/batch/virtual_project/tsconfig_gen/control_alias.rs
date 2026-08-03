//! Exact path mappings that keep Canon's control files private.

use std::path::Path;

use serde_json::{Map, Value};
use vize_carton::cstr;

use super::super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUBS_FILE, PACKAGE_BOUNDARY_FILE,
    SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE,
};

const CONTROL_FILES: &[&str] = &[
    PACKAGE_BOUNDARY_FILE,
    "tsconfig.json",
    "tsconfig.options.json",
    "tsconfig.declaration.json",
    AUTO_IMPORT_STUBS_FILE,
    MODULE_AUGMENTATION_STUBS_FILE,
    VUE_MODULE_STUBS_FILE,
    SHARED_HELPERS_FILE,
];

/// Add exact real-tree mappings wherever a wildcard or exact user target would
/// otherwise expose a same-named control file at the virtual project root.
#[allow(clippy::disallowed_types)]
pub(super) fn protect_control_file_aliases(
    paths: &Map<std::string::String, Value>,
    remapped: &mut Map<std::string::String, Value>,
    project_prefix: &str,
) {
    let mut overrides = Vec::new();
    for (alias, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        for target in targets.iter().filter_map(Value::as_str) {
            for control_file in CONTROL_FILES {
                let Some(exact_alias) = exact_alias_for_target(alias, target, control_file) else {
                    continue;
                };
                if exact_alias != *alias && paths.contains_key(&exact_alias) {
                    continue;
                }
                overrides.push((exact_alias, *control_file));
            }
        }
    }

    for (alias, control_file) in overrides {
        remapped.insert(
            alias,
            Value::Array(vec![Value::String(
                cstr!("{project_prefix}{control_file}").into(),
            )]),
        );
    }
}

#[allow(clippy::disallowed_types)]
fn exact_alias_for_target(alias: &str, target: &str, control_file: &str) -> Option<String> {
    if Path::new(target).is_absolute() {
        return None;
    }
    let target = target.strip_prefix("./").unwrap_or(target);
    let Some((prefix, suffix)) = target.split_once('*') else {
        return (target == control_file).then(|| alias.to_owned());
    };
    let capture = control_file.strip_prefix(prefix)?.strip_suffix(suffix)?;
    alias.contains('*').then(|| alias.replacen('*', capture, 1))
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::protect_control_file_aliases;

    #[allow(clippy::disallowed_types)]
    fn protect(paths: Value) -> Map<std::string::String, Value> {
        let paths = paths.as_object().unwrap();
        let mut remapped = paths.clone();
        protect_control_file_aliases(paths, &mut remapped, "../../../");
        remapped
    }

    #[test]
    fn root_wildcards_cannot_resolve_to_canon_control_files() {
        let protected = protect(json!({ "~/*": ["./*"], "@/*": ["*"] }));
        assert_eq!(
            protected["~/package.json"],
            json!(["../../../package.json"])
        );
        assert_eq!(
            protected["@/tsconfig.json"],
            json!(["../../../tsconfig.json"])
        );
    }

    #[test]
    fn exact_package_target_uses_the_real_project_file() {
        let protected = protect(json!({ "manifest": ["./package.json"] }));
        assert_eq!(protected["manifest"], json!(["../../../package.json"]));
    }

    #[test]
    fn explicit_aliases_and_unrelated_targets_are_unchanged() {
        let protected = protect(json!({
            "~/*": ["./*"],
            "~/package.json": ["fixtures/package.json"],
            "src/*": ["src/*"]
        }));
        assert_eq!(
            protected["~/package.json"],
            json!(["fixtures/package.json"])
        );
        assert_eq!(protected["src/*"], json!(["src/*"]));
    }
}
