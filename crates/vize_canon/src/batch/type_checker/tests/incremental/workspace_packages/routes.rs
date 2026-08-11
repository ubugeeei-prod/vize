use super::super::super::super::{create_project_case, resolve_test_tsgo_binary};
use super::package_binding;
use crate::batch::{BatchTypeChecker, TypeChecker};

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
    let package_root = project_root.join("node_modules/@scope/workspace-vue");
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

    let entry_path = project_root.join("src/entry.ts");
    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    let mut bindings = vec![package_binding(
        &entry_path,
        "@scope/workspace-vue",
        &package_root,
        &component_path,
    )];
    for index in 0..12 {
        let specifier = format!("@scope/unrelated-{index}");
        let root = project_root.join("node_modules").join(&specifier);
        let source = root.join("src/Root.vue");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(
            root.join("package.json"),
            format!("{{\"name\":\"{specifier}\",\"exports\":\"./src/Root.vue\"}}\n"),
        )
        .unwrap();
        std::fs::write(&source, "<template />\n").unwrap();
        bindings.push(package_binding(&entry_path, &specifier, &root, &source));
    }
    checker.set_package_routes(bindings);
    checker.scan_project().expect("initial scan should succeed");
    let clean = checker.check_project().expect("clean check should succeed");
    assert!(clean.diagnostics.is_empty(), "{:#?}", clean.diagnostics);

    for _ in 0..2 {
        let warm = checker
            .check_incremental(std::slice::from_ref(&entry_path))
            .expect("unchanged importer refresh should complete");
        assert!(warm.diagnostics.is_empty(), "{:#?}", warm.diagnostics);
    }
    assert_eq!(checker.package_route_metrics().cache_misses, 0);
    assert_eq!(checker.package_route_metrics().cache_hits, 0);
    assert_eq!(
        checker
            .package_route_metrics()
            .last_refresh_considered_routes,
        0
    );

    std::fs::write(&component_path, clean_source.replace("= 1", "= 'broken'"))
        .expect("broken external SFC should write");
    let broken = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .expect("external SFC incremental check should complete");
    assert_eq!(
        checker.package_route_metrics().last_refresh_total_routes,
        13
    );
    assert_eq!(
        checker
            .package_route_metrics()
            .last_refresh_considered_routes,
        1
    );
    let error = broken
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == component_path && diagnostic.code == Some(2322))
        .unwrap_or_else(|| panic!("missing mapped TS2322: {:#?}", broken.diagnostics));
    assert_eq!((error.line, error.column), (1, 6));
    assert!(
        broken
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2307)),
        "{:#?}",
        broken.diagnostics
    );

    std::fs::write(&component_path, clean_source).unwrap();
    assert!(
        checker
            .check_incremental(std::slice::from_ref(&component_path))
            .unwrap()
            .diagnostics
            .is_empty()
    );
    std::fs::remove_file(&component_path).unwrap();
    let deleted = checker
        .check_incremental(std::slice::from_ref(&component_path))
        .unwrap();
    assert!(
        deleted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(2307))
    );
    std::fs::write(&component_path, clean_source).unwrap();
    assert!(
        checker
            .check_incremental(std::slice::from_ref(&component_path))
            .unwrap()
            .diagnostics
            .is_empty()
    );

    let renamed_path = package_root.join("src/Renamed.vue");
    std::fs::rename(&component_path, &renamed_path).unwrap();
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
    assert!(
        checker
            .check_incremental(&[component_path.clone(), renamed_path])
            .unwrap()
            .diagnostics
            .is_empty()
    );
    assert_eq!(checker.incremental_metrics().session_starts, 1);
    assert_eq!(checker.incremental_metrics().session_reuses, 6);
    assert_eq!(checker.incremental_metrics().session_to_cli_fallbacks, 0);
    assert!(!checker.incremental_metrics().last_full_rebuild);

    let _ = std::fs::remove_dir_all(&project_root);
    let _ = std::fs::remove_dir_all(&package_root);
}
