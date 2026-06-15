use std::fs;

use super::{VirtualProject, unique_case_dir};

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

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let compiler_options = value["compilerOptions"].as_object().unwrap();

    assert_eq!(compiler_options["target"], serde_json::json!("ES2015"));
    assert_eq!(
        compiler_options["moduleResolution"],
        serde_json::json!("bundler")
    );
    assert!(!compiler_options.contains_key("downlevelIteration"));

    let _ = fs::remove_dir_all(&case_dir);
}
