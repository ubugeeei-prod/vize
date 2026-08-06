//! Flattening the `extends` chain into the generated tsconfig.
//!
//! The generated config never `extends` the original, so every inherited option
//! has to be merged here: package-style and relative entries alike, in
//! declaration order, with the extending config winning over all of them.

use std::fs;

use super::{VirtualProject, unique_case_dir};

#[test]
fn materialized_tsconfig_inlines_extends_chain_without_extending_original() {
    let case_dir = unique_case_dir("tsconfig-inline-extends");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(case_dir.join("node_modules/@vue/tsconfig")).unwrap();
    fs::write(
        case_dir.join("node_modules/@vue/tsconfig/tsconfig.dom.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "jsx": "preserve",
    "moduleResolution": "bundler"
  }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{
  "compilerOptions": {
    "noUnusedLocals": true,
    "baseUrl": "."
  }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": ["@vue/tsconfig/tsconfig.dom.json", "./tsconfig.base.json"],
  "compilerOptions": {
    "jsx": "react-jsx"
  },
  "files": ["src/real-tree-only.ts"]
}"#,
    )
    .unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();

    // The original tsconfig is never `extends`-ed: Corsa would re-parse the
    // whole chain and fail the CLI run on config diagnostics for options the
    // mirror already strips (e.g. the removed `baseUrl`), and the real tree's
    // `files` list must not leak into the virtual program.
    assert!(value.get("extends").is_none());

    let compiler_options = value["compilerOptions"].as_object().unwrap();
    // Inherited through the package-style extends entry.
    assert_eq!(compiler_options["strict"], serde_json::Value::Bool(true));
    assert_eq!(
        compiler_options["moduleResolution"],
        serde_json::json!("bundler")
    );
    // Inherited through the relative extends entry.
    assert_eq!(
        compiler_options["noUnusedLocals"],
        serde_json::Value::Bool(true)
    );
    // The extending config wins over every extends entry.
    assert_eq!(compiler_options["jsx"], serde_json::json!("react-jsx"));
    // Path-sensitive options stay stripped.
    assert!(!compiler_options.contains_key("baseUrl"));

    let _ = fs::remove_dir_all(&case_dir);
}

/// Each `extends` array entry is flattened on its own before the entries are
/// merged, so an ancestor two entries share is inherited by both. The later
/// entry inherits the shared value and overrides the earlier entry's own
/// override; short-circuiting the second visit of the shared config would leave
/// the earlier override standing instead.
#[test]
fn a_shared_ancestor_is_inherited_by_every_sibling_extends_entry() {
    let case_dir = unique_case_dir("tsconfig-extends-diamond");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.shared.json"),
        r#"{ "compilerOptions": { "noUnusedLocals": true } }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.first.json"),
        r#"{
  "extends": "./tsconfig.shared.json",
  "compilerOptions": { "noUnusedLocals": false }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.second.json"),
        r#"{
  "extends": "./tsconfig.shared.json",
  "compilerOptions": { "noUnusedParameters": true }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": ["./tsconfig.first.json", "./tsconfig.second.json"],
  "include": ["src"]
}"#,
    )
    .unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let compiler_options = value["compilerOptions"].as_object().unwrap();

    assert_eq!(
        compiler_options["noUnusedLocals"],
        serde_json::Value::Bool(true),
        "the second entry's inherited value must win: {compiler_options:?}"
    );
    assert_eq!(
        compiler_options["noUnusedParameters"],
        serde_json::Value::Bool(true)
    );

    let _ = fs::remove_dir_all(&case_dir);
}
