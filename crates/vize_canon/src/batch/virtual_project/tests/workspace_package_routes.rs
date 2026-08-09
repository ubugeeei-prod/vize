use std::fs;

use super::{VirtualProject, unique_case_dir};

#[test]
fn a_workspace_package_route_precedes_real_tree_path_fallbacks() {
    let project_root = unique_case_dir("workspace-package-route");
    let package_root = project_root.parent().unwrap().join(
        vize_carton::cstr!(
            "{}-package",
            project_root.file_name().unwrap().to_string_lossy()
        )
        .as_str(),
    );
    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "paths": {
      "@scope/workspace-vue": ["./missing"]
    }
  }
}"#,
    )
    .unwrap();
    let entry_path = project_root.join("src/entry.ts");
    fs::write(
        &entry_path,
        "import Widget from '@scope/workspace-vue'\nvoid Widget\n",
    )
    .unwrap();
    let component_path = package_root.join("src/Root.vue");
    fs::write(
        &component_path,
        "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&project_root).unwrap();
    project.set_virtual_module_aliases([("@scope/workspace-vue".into(), component_path.clone())]);
    project.register_path(&entry_path).unwrap();
    project.register_virtual_module_alias_targets().unwrap();
    project.materialize().unwrap();

    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(project.virtual_root().join("tsconfig.json")).unwrap(),
    )
    .unwrap();
    let targets = config["compilerOptions"]["paths"]["@scope/workspace-vue"]
        .as_array()
        .unwrap();
    let routed = targets[0].as_str().unwrap();
    assert!(routed.starts_with("./__vize_external__/"), "{targets:#?}");
    assert!(routed.ends_with("/Root.vue.ts"), "{targets:#?}");
    assert!(
        targets
            .iter()
            .skip(1)
            .filter_map(serde_json::Value::as_str)
            .any(|target| target.contains("missing")),
        "the user's fallback must remain after the virtual route: {targets:#?}"
    );
    let entry = project.find_by_original(&entry_path).unwrap();
    assert!(entry.content.contains("'@scope/workspace-vue'"));
    assert!(!entry.content.contains("__vize_external__"));

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&package_root);
}
