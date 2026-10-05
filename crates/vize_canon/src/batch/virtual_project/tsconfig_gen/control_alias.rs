//! Exact path mappings that keep Canon's control files private.

use std::{borrow::Cow, path::Path};

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
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(super) fn protect_control_file_aliases(
    paths: &Map<std::string::String, Value>,
    remapped: &mut Map<std::string::String, Value>,
    project_prefix: &str,
    project_root: &Path,
    shared_helpers: &str,
) {
    let mut overrides = Vec::new();
    for (alias, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        for target in targets.iter().filter_map(Value::as_str) {
            let Some(target) = target_onto_project_root(target, project_root) else {
                continue;
            };
            for control_file in CONTROL_FILES
                .iter()
                .copied()
                .chain(std::iter::once(shared_helpers))
            {
                let Some(exact_alias) = exact_alias_for_target(alias, &target, control_file) else {
                    continue;
                };
                if exact_alias != *alias && paths.contains_key(&exact_alias) {
                    continue;
                }
                overrides.push((exact_alias, control_file));
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

// Mirror expansion also translates absolute in-project targets onto the root.
// Use the identical lexical boundary here; unrelated absolute targets remain
// direct authored imports and cannot expose a private mirror control.
fn target_onto_project_root<'a>(target: &'a str, project_root: &Path) -> Option<Cow<'a, str>> {
    let path = Path::new(target);
    if !path.is_absolute() {
        return Some(Cow::Borrowed(target));
    }
    let normalized = super::super::tsconfig_paths::normalize_path_lexically(path);
    let relative = normalized.strip_prefix(project_root).ok()?;
    Some(Cow::Owned(relative.to_string_lossy().replace('\\', "/")))
}

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
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
    use std::path::Path;

    use super::protect_control_file_aliases;

    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    fn protect(paths: Value) -> Map<std::string::String, Value> {
        let paths = paths.as_object().unwrap();
        let mut remapped = paths.clone();
        protect_control_file_aliases(
            paths,
            &mut remapped,
            "../../../",
            Path::new("/source"),
            "apps/web/__vize_helpers.d.ts",
        );
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

    #[test]
    fn nested_helpers_protect_wildcard_exact_absolute_targets_and_authored_priority() {
        let root = std::env::temp_dir().join("vize-helper-root");
        let foreign = std::env::temp_dir().join("vize-helper-other/apps/web/__vize_helpers.d.ts");
        let paths = json!({
            "private/*": ["apps/web/*"],
            "helper": ["./apps/web/__vize_helpers.d.ts"],
            "absolute": [root.join("apps/web/__vize_helpers.d.ts")],
            "authored/*": ["apps/web/*"],
            "authored/__vize_helpers.d.ts": ["apps/web/authored.ts"],
            "foreign": [foreign]
        });
        let paths = paths.as_object().unwrap();
        let mut protected = paths.clone();
        protect_control_file_aliases(
            paths,
            &mut protected,
            "../../../",
            &root,
            "apps/web/__vize_helpers.d.ts",
        );
        for alias in ["private/__vize_helpers.d.ts", "helper", "absolute"] {
            assert_eq!(
                protected[alias],
                json!(["../../../apps/web/__vize_helpers.d.ts"])
            );
        }
        assert_eq!(
            protected["authored/__vize_helpers.d.ts"],
            json!(["apps/web/authored.ts"])
        );
        assert_eq!(protected["foreign"], json!([foreign]));
    }
}
