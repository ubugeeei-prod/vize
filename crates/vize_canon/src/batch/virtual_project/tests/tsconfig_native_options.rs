use std::fs;

use super::{SHARED_HELPERS_FILE, VirtualProject, unique_case_dir};

#[test]
fn materialized_tsconfig_normalizes_native_removed_options() {
    let case_dir = unique_case_dir("tsconfig-native-removed-options");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES5",
    "module": "ESNext",
    "moduleResolution": "node",
    "downlevelIteration": true
  }
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

    assert_eq!(compiler_options["target"], serde_json::json!("ES2015"));
    assert_eq!(
        compiler_options["moduleResolution"],
        serde_json::json!("bundler")
    );
    assert_eq!(
        compiler_options["resolvePackageJsonExports"],
        serde_json::json!(false)
    );
    assert_eq!(
        compiler_options["resolvePackageJsonImports"],
        serde_json::json!(false)
    );
    assert!(!compiler_options.contains_key("downlevelIteration"));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialized_tsconfig_adds_vue_jsx_defaults_for_lowered_tsx() {
    let case_dir = unique_case_dir("tsconfig-lowered-tsx-jsx");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let tsx_path = src_dir.join("Comp.tsx");
    fs::write(
        &tsx_path,
        "export const Comp = () => <button class=\"primary\">Save</button>;\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
    project.set_jsx_typecheck(true);
    project.register_path(&tsx_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = project.virtual_root().join("tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let compiler_options = value["compilerOptions"].as_object().unwrap();

    assert_eq!(
        compiler_options["jsx"],
        serde_json::json!("preserve"),
        "{value:#}"
    );
    assert_eq!(
        compiler_options["jsxImportSource"],
        serde_json::json!("vue"),
        "{value:#}"
    );
    let helpers = fs::read_to_string(project.virtual_root().join(SHARED_HELPERS_FILE)).unwrap();
    assert!(
        helpers.contains("/// <reference types=\"vue/jsx\" />"),
        "{helpers}"
    );

    let _ = fs::remove_dir_all(&case_dir);
}
