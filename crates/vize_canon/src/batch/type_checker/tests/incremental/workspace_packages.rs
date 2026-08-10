use super::{BatchTypeChecker, TypeChecker, create_project_case, resolve_test_tsgo_binary};

#[test]
fn refreshes_workspace_package_vue_routes_in_the_persistent_session() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "incremental-workspace-package-vue",
        &[(
            "src/entry.ts",
            r#"import Widget from '@scope/workspace-vue'
type IsAny<T> = 0 extends 1 & T ? true : false
const componentMustBeTyped: IsAny<typeof Widget> = false
void componentMustBeTyped
"#,
        )],
    );
    let package_root = project_root.parent().unwrap().join(
        vize_carton::cstr!(
            "{}-package",
            project_root.file_name().unwrap().to_string_lossy()
        )
        .as_str(),
    );
    let _ = std::fs::remove_dir_all(&package_root);
    std::fs::create_dir_all(package_root.join("src")).unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/workspace-vue",
  "exports": {
    ".": { "types": "./src/Root.vue", "default": "./src/Root.vue" }
  }
}"#,
    )
    .unwrap();
    let component_path = package_root.join("src/Root.vue");
    let clean_source = r#"<script setup lang="ts">
const count: number = 1
</script>
<template>{{ count }}</template>
"#;
    std::fs::write(&component_path, clean_source).unwrap();

    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker.set_virtual_module_aliases([("@scope/workspace-vue".into(), component_path.clone())]);
    checker.scan_project().expect("initial scan should succeed");
    let clean = checker.check_project().expect("clean check should succeed");
    assert!(
        clean.diagnostics.is_empty(),
        "workspace package route was not clean: {:#?}",
        clean.diagnostics
    );

    std::fs::write(&component_path, clean_source.replace("= 1", "= 'broken'"))
        .expect("broken external SFC should write");
    let broken = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .expect("external SFC incremental check should complete");
    let error = broken
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == component_path && diagnostic.code == Some(2322))
        .unwrap_or_else(|| {
            panic!(
                "external SFC patch did not report mapped TS2322: {:#?}",
                broken.diagnostics
            )
        });
    assert_eq!((error.line, error.column), (1, 6));
    assert!(
        broken
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2307)),
        "persistent route regressed to TS2307: {:#?}",
        broken.diagnostics
    );

    std::fs::write(&component_path, clean_source).expect("external SFC repair should write");
    let repaired = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .expect("external SFC repair should complete");
    assert!(
        repaired.diagnostics.is_empty(),
        "external SFC repair retained stale diagnostics: {:#?}",
        repaired.diagnostics
    );

    std::fs::remove_file(&component_path).expect("external SFC delete should succeed");
    let deleted = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .expect("external SFC delete should refresh the session");
    assert!(
        deleted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2307)),
        "a deleted package target must invalidate its importer: {:#?}",
        deleted.diagnostics
    );

    std::fs::write(&component_path, clean_source).expect("external SFC recreate should write");
    let recreated = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .expect("known external SFC recreate should refresh the session");
    assert!(
        recreated.diagnostics.is_empty(),
        "external SFC recreate retained stale diagnostics: {:#?}",
        recreated.diagnostics
    );

    let renamed_path = package_root.join("src/Renamed.vue");
    std::fs::rename(&component_path, &renamed_path).expect("external SFC rename should succeed");
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/workspace-vue",
  "exports": {
    ".": { "types": "./src/Renamed.vue", "default": "./src/Renamed.vue" }
  }
}"#,
    )
    .unwrap();
    checker.set_virtual_module_aliases([("@scope/workspace-vue".into(), renamed_path.clone())]);
    let renamed = checker
        .check_incremental(&[component_path.clone(), renamed_path])
        .expect("updated external package route should refresh the session");
    assert!(
        renamed.diagnostics.is_empty(),
        "external SFC rename retained stale diagnostics: {:#?}",
        renamed.diagnostics
    );
    assert_eq!(checker.incremental_metrics().session_starts, 1);
    assert_eq!(checker.incremental_metrics().session_reuses, 4);

    let _ = std::fs::remove_dir_all(&project_root);
    let _ = std::fs::remove_dir_all(&package_root);
}
