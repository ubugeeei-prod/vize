use std::fs;

use super::{TsconfigInputCache, collect_default_check_files, unique_case_dir};

#[test]
fn allow_js_is_inherited_and_can_be_disabled_by_a_child_config() {
    let case_dir = unique_case_dir("tsconfig-allow-js-extends");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/main.js"), "export const value = 1;").unwrap();
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{ "compilerOptions": { "allowJs": true } }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    )
    .unwrap();

    assert_eq!(
        collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json"))),
        vec![case_dir.join("src/main.js")]
    );

    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "allowJs": false },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    assert!(
        collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json"))).is_empty()
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn explicit_collection_detects_allow_js_in_referenced_projects() {
    let case_dir = unique_case_dir("tsconfig-allow-js-reference");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("packages/app")).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": [], "references": [{ "path": "packages/app" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/app/tsconfig.json"),
        r#"{ "compilerOptions": { "allowJs": true }, "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let mut cache = TsconfigInputCache::default();
    assert!(cache.project_graph_allows_javascript(Some(&case_dir.join("tsconfig.json"))));
    assert!(!cache.project_graph_allows_javascript(None));

    let _ = fs::remove_dir_all(&case_dir);
}
