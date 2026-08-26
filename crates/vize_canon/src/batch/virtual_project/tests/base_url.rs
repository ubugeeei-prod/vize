//! `baseUrl` emulation in the generated tsconfig (#3886).
//!
//! The native checker removed `baseUrl`; these tests pin the two halves of the
//! emulation: a synthesized `"*"` alias keeps bare specifiers resolving, and
//! relative `paths` targets anchor to the effective `baseUrl` the way
//! TypeScript 5.x/6.x resolve them — not to the declaring config's directory.

use std::fs;
use std::path::Path;

use super::{VirtualProject, unique_case_dir};
use crate::batch::project_virtual_root;

fn generated_paths(case_dir: &Path) -> serde_json::Map<String, serde_json::Value> {
    let tsconfig_path = project_virtual_root(case_dir).join("tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    value["compilerOptions"]["paths"]
        .as_object()
        .cloned()
        .unwrap_or_default()
}

fn materialize_with_tsconfig(case_dir: &Path, tsconfig: &str) {
    let _ = fs::remove_dir_all(case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(case_dir.join("tsconfig.json"), tsconfig).unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();
}

#[test]
fn a_root_base_url_synthesizes_the_wildcard_alias() {
    let case_dir = unique_case_dir("base-url-root-wildcard");
    materialize_with_tsconfig(
        &case_dir,
        r#"{
  "compilerOptions": {
    "baseUrl": "."
  }
}"#,
    );

    // Mirror candidate, real-tree fallback, `.vue.ts` mirror candidate — the
    // same triple every user alias gets, so bare specifiers resolve generated
    // SFC modules and out-of-mirror sources alike.
    assert_eq!(
        generated_paths(&case_dir)["*"],
        serde_json::json!(["./*", "../../../../*", "./*.vue.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn a_nested_base_url_anchors_both_wildcard_and_paths_targets() {
    let case_dir = unique_case_dir("base-url-nested-anchor");
    materialize_with_tsconfig(
        &case_dir,
        r#"{
  "compilerOptions": {
    "baseUrl": "./src",
    "paths": {
      "@lib/*": ["lib/*"]
    }
  }
}"#,
    );

    let paths = generated_paths(&case_dir);
    assert_eq!(
        paths["*"],
        serde_json::json!(["./src/*", "../../../../src/*", "./src/*.vue.ts"])
    );
    // TypeScript resolves a relative `paths` target against `baseUrl`, so
    // `lib/*` means `src/lib/*` here — anchoring it to the tsconfig directory
    // would silently point the alias one level too high.
    assert_eq!(
        paths["@lib/*"],
        serde_json::json!(["./src/lib/*", "../../../../src/lib/*", "./src/lib/*.vue.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn a_user_declared_wildcard_is_not_overwritten() {
    let case_dir = unique_case_dir("base-url-user-wildcard");
    materialize_with_tsconfig(
        &case_dir,
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "*": ["vendor/*"]
    }
  }
}"#,
    );

    assert_eq!(
        generated_paths(&case_dir)["*"],
        serde_json::json!(["./vendor/*", "../../../../vendor/*", "./vendor/*.vue.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn without_base_url_no_wildcard_is_synthesized() {
    let case_dir = unique_case_dir("base-url-absent");
    materialize_with_tsconfig(
        &case_dir,
        r#"{
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}"#,
    );

    let paths = generated_paths(&case_dir);
    assert!(!paths.contains_key("*"), "{paths:?}");

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn an_inherited_base_url_anchors_paths_declared_by_the_extending_config() {
    let case_dir = unique_case_dir("base-url-extends-anchor");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("config")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    // The parent declares `baseUrl` relative to *its own* directory; the child
    // declares `paths`. TypeScript anchors the child's targets to the parent's
    // baseUrl — `config/` — not to the child's directory.
    fs::write(
        case_dir.join("config/tsconfig.base.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": "."
  }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r##"{
  "extends": "./config/tsconfig.base.json",
  "compilerOptions": {
    "paths": {
      "#shared/*": ["shared/*"]
    }
  }
}"##,
    )
    .unwrap();
    let vue_path = case_dir.join("src/App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let paths = generated_paths(&case_dir);
    assert_eq!(
        paths["#shared/*"],
        serde_json::json!([
            "./config/shared/*",
            "../../../../config/shared/*",
            "./config/shared/*.vue.ts"
        ])
    );
    assert_eq!(
        paths["*"],
        serde_json::json!(["./config/*", "../../../../config/*", "./config/*.vue.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn a_child_base_url_reanchors_paths_declared_by_the_extended_config() {
    let case_dir = unique_case_dir("base-url-child-override-anchor");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("config")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(
        case_dir.join("config/tsconfig.base.json"),
        r##"{
  "compilerOptions": {
    "paths": {
      "#shared/*": ["shared/*"]
    }
  }
}"##,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./config/tsconfig.base.json",
  "compilerOptions": {
    "baseUrl": "./src"
  }
}"#,
    )
    .unwrap();
    let vue_path = case_dir.join("src/App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let paths = generated_paths(&case_dir);
    assert_eq!(
        paths["#shared/*"],
        serde_json::json!([
            "./src/shared/*",
            "../../../../src/shared/*",
            "./src/shared/*.vue.ts"
        ])
    );
    assert_eq!(
        paths["*"],
        serde_json::json!(["./src/*", "../../../../src/*", "./src/*.vue.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}
