//! Solution-style tsconfig handling (#3915).
//!
//! The create-vue default layout ships a references-only shell —
//! `{ "files": [], "references": [{ "path": "./tsconfig.app.json" }, …] }` —
//! with every compiler option, including `paths`, living in the referenced
//! project configs. An anchor that reads only the shell resolves no aliases,
//! so consumers that found no `paths` in the anchored chain retry through the
//! configs the shell references.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::super::tsconfig_paths::{normalize_path_lexically, parse_jsonc_value};

/// The project configs referenced by `tsconfig_path`, in declaration order.
///
/// A reference `path` may name a config file or a directory (TypeScript
/// resolves a directory to its `tsconfig.json`); entries that do not resolve
/// to an existing file are dropped. A config without references — or one that
/// cannot be read — yields an empty list.
pub(in super::super) fn referenced_project_configs(tsconfig_path: &Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(tsconfig_path) else {
        return Vec::new();
    };
    let Ok(config) = parse_jsonc_value(&content) else {
        return Vec::new();
    };
    let Some(references) = config.get("references").and_then(Value::as_array) else {
        return Vec::new();
    };
    let base = tsconfig_path.parent().unwrap_or(Path::new("."));
    references
        .iter()
        .filter_map(|reference| reference.get("path").and_then(Value::as_str))
        .filter_map(|path| {
            let joined = normalize_path_lexically(&base.join(path));
            if joined.is_file() {
                return Some(joined);
            }
            let as_directory = joined.join("tsconfig.json");
            as_directory.is_file().then_some(as_directory)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn write(root: &Path, name: &str, content: &str) {
        std::fs::write(root.join(name), content).unwrap();
    }

    #[test]
    fn file_and_directory_references_resolve_missing_ones_drop() {
        let root = std::env::temp_dir().join(format!("vize-refs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("sub")).unwrap();
        write(
            &root,
            "tsconfig.json",
            r#"{
  // create-vue style shell
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./sub" },
    { "path": "./missing.json" }
  ]
}"#,
        );
        write(&root, "tsconfig.app.json", "{}");
        write(&root.join("sub"), "tsconfig.json", "{}");

        let configs = super::referenced_project_configs(&root.join("tsconfig.json"));
        let names: Vec<_> = configs
            .iter()
            .map(|config| {
                config
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(names, ["tsconfig.app.json", "sub/tsconfig.json"]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_config_without_references_yields_nothing() {
        let root = std::env::temp_dir().join(format!("vize-norefs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        write(&root, "tsconfig.json", r#"{ "compilerOptions": {} }"#);
        assert!(super::referenced_project_configs(&root.join("tsconfig.json")).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
