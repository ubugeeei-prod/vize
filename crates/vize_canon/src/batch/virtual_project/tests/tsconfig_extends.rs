//! Flattening the `extends` chain into the generated tsconfig.
//!
//! The generated config never `extends` the original, so every inherited option
//! has to be merged here: package-style and relative entries alike, in
//! declaration order, with the extending config winning over all of them.

use std::fs;

use super::{VirtualProject, unique_case_dir};
use crate::batch::snapshot_tsconfig_compiler_options;

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

    let tsconfig_path = project.virtual_root().join("tsconfig.json");
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

    let tsconfig_path = project.virtual_root().join("tsconfig.json");
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

#[test]
fn external_snapshot_is_flattened_and_cache_location_independent() {
    let case_dir = unique_case_dir("tsconfig-external-snapshot");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("node_modules/example-config")).unwrap();
    fs::create_dir_all(case_dir.join("src/types")).unwrap();
    fs::write(
        case_dir.join("node_modules/example-config/base.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "baseUrl": "../../src",
    "rootDirs": ["./generated", "../../src"],
    "outDir": "./dist",
    "mapRoot": "./maps",
    "sourceRoot": "https://cdn.example.test/sources/"
  }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r##"{
  "extends": "example-config/base.json",
  "compilerOptions": { "paths": { "~/*": ["types/*"] } }
}"##,
    )
    .unwrap();

    let options =
        snapshot_tsconfig_compiler_options(&case_dir, &case_dir.join("tsconfig.json")).unwrap();
    assert_eq!(options["strict"], serde_json::json!(true));
    assert_eq!(
        options["baseUrl"],
        serde_json::json!(case_dir.join("src").to_string_lossy())
    );
    assert_eq!(
        options["paths"]["~/*"],
        serde_json::json!([case_dir.join("src/types/*").to_string_lossy()])
    );
    assert_eq!(
        options["rootDirs"],
        serde_json::json!([
            case_dir.join("node_modules/example-config/generated"),
            case_dir.join("src")
        ])
    );
    assert_eq!(
        options["outDir"],
        serde_json::json!(case_dir.join("node_modules/example-config/dist"))
    );
    assert_eq!(
        options["mapRoot"],
        serde_json::json!(case_dir.join("node_modules/example-config/maps"))
    );
    assert_eq!(
        options["sourceRoot"],
        serde_json::json!("https://cdn.example.test/sources/")
    );

    let _ = fs::remove_dir_all(&case_dir);
}

/// A bare `extends` specifier resolves through Node resolution, which follows
/// symlinks, so the relative options the extended config declares anchor to the
/// config's real directory rather than to the `node_modules` entry that pointed
/// at it. Under pnpm every workspace dependency is such a link, and anchoring to
/// the link walked `../..` back out through `node_modules` and produced a
/// `node_modules/node_modules/@types` root that cannot exist — a TS2688 that
/// suppresses every per-file diagnostic in the run (#4425).
#[cfg(unix)]
#[test]
fn relative_type_roots_from_a_linked_package_anchor_to_the_real_directory() {
    let case_dir = unique_case_dir("tsconfig-linked-type-roots");
    let _ = fs::remove_dir_all(&case_dir);
    let root = case_dir.join("workspace");
    let shared = root.join("packages/shared-tsconfig/base");
    let app = root.join("apps/app");
    fs::create_dir_all(&shared).unwrap();
    fs::create_dir_all(app.join("src")).unwrap();
    fs::create_dir_all(root.join("node_modules/@types/node")).unwrap();
    fs::write(
        root.join("node_modules/@types/node/index.d.ts"),
        "export {};",
    )
    .unwrap();

    // `../../../node_modules/@types` from the shared package's own directory is
    // the workspace root's `@types`.
    fs::write(
        shared.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "typeRoots": ["../../../node_modules/@types"],
    "types": ["node"]
  }
}"#,
    )
    .unwrap();

    // pnpm's shape: the dependency is a link into the workspace, not a copy.
    let app_modules = app.join("node_modules/@repro");
    fs::create_dir_all(&app_modules).unwrap();
    std::os::unix::fs::symlink(
        root.join("packages/shared-tsconfig"),
        app_modules.join("shared-tsconfig"),
    )
    .unwrap();

    fs::write(
        app.join("tsconfig.json"),
        r#"{
  "extends": "@repro/shared-tsconfig/base/tsconfig.json",
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let vue_path = app.join("src/App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&app).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = project.virtual_root().join("tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let type_roots = value["compilerOptions"]["typeRoots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    for entry in &type_roots {
        assert!(
            !entry.contains("node_modules/node_modules"),
            "typeRoots must not walk back out through node_modules: {type_roots:?}",
        );
    }
    // The root sits outside the app, so it stays absolute and is listed once —
    // `remap_dir_entries` only pairs a mirror copy with a real-tree fallback for
    // entries that live under the project root.
    let expected = root.join("node_modules/@types");
    assert_eq!(
        type_roots,
        vec![
            vize_carton::path::canonicalize_non_verbatim(&expected)
                .to_string_lossy()
                .into_owned()
        ],
        "expected the workspace root's @types",
    );

    let _ = fs::remove_dir_all(&case_dir);
}
