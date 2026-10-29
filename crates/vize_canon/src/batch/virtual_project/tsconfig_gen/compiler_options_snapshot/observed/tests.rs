use super::CompilerOptionsSnapshot;
use crate::snapshot_tsconfig_compiler_options;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

#[test]
fn compiler_options_snapshot_preserves_files_and_ordered_effective_options() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let app = root.join("app");
    std::fs::create_dir(&app).unwrap();
    let config = app.join("tsconfig.json");
    std::fs::write(
        root.join("base.json"),
        r#"{
        "compilerOptions": {
            "baseUrl": ".",
            "paths": { "@/*": ["src/*", "fallback/*"] },
            "moduleSuffixes": [".native", ""],
            "rootDirs": ["src", "generated"],
            "customConditions": ["development", "browser"]
        }
    }"#,
    )
    .unwrap();
    std::fs::write(
        &config,
        r#"{
        "extends": "../base.json", "compilerOptions": { "strict": true }
    }"#,
    )
    .unwrap();
    let before = inventory(&root);
    let map = snapshot_tsconfig_compiler_options(&app, &config).unwrap();
    let captured = CompilerOptionsSnapshot::capture(&app, Some(&config), &[]).unwrap();
    assert_eq!(map, *captured.options());
    assert_eq!(
        map,
        serde_json::json!({
            "baseUrl": root.to_string_lossy().replace('\\', "/"),
            "paths": {"@/*": [
                root.join("src/*").to_string_lossy().replace('\\', "/"),
                root.join("fallback/*").to_string_lossy().replace('\\', "/")
            ]},
            "moduleSuffixes": [".native", ""],
            "rootDirs": [
                root.join("src").to_string_lossy().replace('\\', "/"),
                root.join("generated").to_string_lossy().replace('\\', "/")
            ],
            "customConditions": ["development", "browser"],
            "strict": true
        })
        .as_object()
        .unwrap()
        .clone()
    );
    assert_eq!(captured.project_root(), app.as_path());
    assert_eq!(captured.tsconfig_path(), Some(config.as_path()));
    assert!(captured.unresolved_extends().is_empty());
    assert!(captured.observations().changed_paths().is_empty());
    assert_eq!(
        inventory(&root),
        before,
        "reading options must not create a virtual cache"
    );
}

#[test]
fn compiler_options_snapshot_records_absence_and_rejects_later_config() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let candidates = [root.join("tsconfig.json"), root.join("jsconfig.json")];
    assert!(CompilerOptionsSnapshot::capture(&root, None, &[]).is_err());
    let captured = CompilerOptionsSnapshot::capture(&root, None, &candidates).unwrap();
    assert_eq!(captured.tsconfig_path(), None);
    assert!(captured.options().is_empty());
    assert_eq!(
        captured.observations().paths(),
        vec![candidates[1].clone(), candidates[0].clone()]
    );
    assert!(inventory(&root).is_empty());
    std::fs::write(&candidates[1], "{}").unwrap();
    assert_eq!(
        captured.observations().changed_paths(),
        vec![candidates[1].clone()]
    );
    assert!(CompilerOptionsSnapshot::capture(&root, None, &candidates).is_err());

    #[cfg(unix)]
    {
        let linked_root = root.join("linked");
        std::fs::create_dir(&linked_root).unwrap();
        let missing_target = root.join("future.json");
        let config_link = linked_root.join("tsconfig.json");
        std::os::unix::fs::symlink(&missing_target, &config_link).unwrap();
        assert!(
            CompilerOptionsSnapshot::capture(
                &linked_root,
                None,
                std::slice::from_ref(&config_link)
            )
            .is_err(),
            "a dangling config entry is unreadable, not absent"
        );
        std::fs::write(&missing_target, "{}").unwrap();
        assert!(
            CompilerOptionsSnapshot::capture(
                &linked_root,
                None,
                std::slice::from_ref(&config_link)
            )
            .is_err()
        );
        let selected =
            CompilerOptionsSnapshot::capture(&linked_root, Some(&config_link), &[]).unwrap();
        assert!(selected.options().is_empty());
        assert!(selected.observations().changed_paths().is_empty());
        std::fs::write(&missing_target, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
        assert_eq!(
            selected.observations().changed_paths(),
            vec![config_link.clone()]
        );
        let repaired =
            CompilerOptionsSnapshot::capture(&linked_root, Some(&config_link), &[]).unwrap();
        assert_eq!(
            repaired.options(),
            serde_json::json!({"strict":true}).as_object().unwrap()
        );
        assert!(selected.options().is_empty());
    }
}

#[test]
fn compiler_options_snapshot_selected_config_retains_higher_priority_absence() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let missing = root.join("tsconfig.json");
    let selected = root.join("jsconfig.json");
    std::fs::write(&selected, "{}").unwrap();
    let captured =
        CompilerOptionsSnapshot::capture(&root, Some(&selected), std::slice::from_ref(&missing))
            .unwrap();
    assert_eq!(
        captured.options(),
        serde_json::json!({
            "allowJs": true, "maxNodeModuleJsDepth": 2, "skipLibCheck": true, "noEmit": true
        })
        .as_object()
        .unwrap()
    );
    std::fs::write(&missing, "{}").unwrap();
    assert_eq!(
        captured.observations().changed_paths(),
        vec![missing.clone()]
    );
    assert!(
        CompilerOptionsSnapshot::capture(&root, Some(&selected), std::slice::from_ref(&missing),)
            .is_err()
    );
}

#[test]
fn compiler_options_snapshot_never_converts_failed_reads_or_parses_to_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err());
    std::fs::write(&config, "{invalid").unwrap();
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err());
    let base = root.join("base.json");
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&base, "{invalid").unwrap();
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err());
    std::fs::write(&base, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
    let captured = CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).unwrap();
    assert_eq!(
        captured.options(),
        serde_json::json!({"strict":true}).as_object().unwrap()
    );
}

#[test]
fn compiler_options_snapshot_retains_missing_extends_and_original_revision() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    let missing = root.join("missing.json");
    std::fs::write(
        &config,
        r#"{
        "extends":"./missing.json", "compilerOptions":{"strict":true}
    }"#,
    )
    .unwrap();
    let captured = CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).unwrap();
    assert_eq!(
        captured.unresolved_extends(),
        &[crate::UnresolvedExtends {
            config: config.clone(),
            specifier: "./missing.json".into(),
        }]
    );
    assert_eq!(
        captured.options(),
        serde_json::json!({"strict":true}).as_object().unwrap()
    );
    assert_eq!(
        captured.observations().paths(),
        vec![missing.clone(), config.clone()]
    );
    std::fs::write(
        &missing,
        r#"{"compilerOptions":{"moduleSuffixes":[".native",""]}}"#,
    )
    .unwrap();
    assert_eq!(
        captured.observations().changed_paths(),
        vec![missing.clone()]
    );
    let repaired = CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).unwrap();
    assert!(repaired.unresolved_extends().is_empty());
    assert_eq!(
        repaired.options(),
        serde_json::json!({
            "strict":true,"moduleSuffixes":[".native",""]
        })
        .as_object()
        .unwrap()
    );
    assert_eq!(
        captured.options(),
        serde_json::json!({"strict":true}).as_object().unwrap()
    );
}

#[test]
fn compiler_options_snapshot_requires_absolute_host_identity() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    assert!(CompilerOptionsSnapshot::capture(Path::new("relative"), Some(&config), &[]).is_err());
    assert!(
        CompilerOptionsSnapshot::capture(&root, Some(Path::new("tsconfig.json")), &[]).is_err()
    );
    assert!(
        CompilerOptionsSnapshot::capture(&root, None, &[PathBuf::from("jsconfig.json")]).is_err()
    );
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_ok());
    let logical = root.join("missing-parent/../tsconfig.json");
    let normalized = CompilerOptionsSnapshot::capture(&root, Some(&logical), &[]).unwrap();
    assert_eq!(normalized.tsconfig_path(), Some(config.as_path()));
    assert!(
        CompilerOptionsSnapshot::capture(&root, None, std::slice::from_ref(&logical)).is_err(),
        "absence and the loader must observe the same normalized path"
    );
}

#[test]
fn compiler_options_snapshot_refuses_erased_invalid_shapes_and_retains_valid_nulls() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    for source in [
        "[]",
        "null",
        r#"{"compilerOptions":"wrong"}"#,
        r#"{"extends":null}"#,
        r#"{"extends":1}"#,
        r#"{"extends":[null]}"#,
    ] {
        std::fs::write(&config, source).unwrap();
        assert!(
            CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err(),
            "{source}"
        );
    }
    for source in [
        r#"{"compilerOptions":null}"#,
        r#"{"compilerOptions":{"moduleSuffixes":null,"paths":null,"rootDirs":null,"customConditions":null}}"#,
    ] {
        std::fs::write(&config, source).unwrap();
        assert!(
            CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_ok(),
            "{source}"
        );
    }
    let base = root.join("base.json");
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&base, r#"{"compilerOptions":"wrong"}"#).unwrap();
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err());
    std::fs::write(&base, r#"{"extends":"./tsconfig.json"}"#).unwrap();
    assert!(CompilerOptionsSnapshot::capture(&root, Some(&config), &[]).is_err());
}
