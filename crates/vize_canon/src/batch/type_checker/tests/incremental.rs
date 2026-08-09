use super::super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

#[path = "incremental/file_lifecycle.rs"]
mod file_lifecycle;

#[test]
fn observes_broken_and_repaired_vue_patch() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let clean_source = r#"<script setup lang="ts">
const total: number = 1
</script>

<template><p>{{ total }}</p></template>
"#;
    let broken_source = clean_source.replace("= 1", "= 'broken'");
    let project_root =
        create_project_case("incremental-vue-patch", &[("src/App.vue", clean_source)]);
    let app_path = project_root.join("src/App.vue");

    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker.scan_project().expect("initial scan should succeed");

    let clean = checker.check_project().expect("clean check should succeed");
    assert!(
        clean
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322)),
        "clean source unexpectedly reported TS2322: {:#?}",
        clean.diagnostics
    );

    std::fs::write(&app_path, &broken_source).expect("broken patch should write");
    let broken = checker
        .check_incremental(std::slice::from_ref(&app_path))
        .expect("broken incremental check should complete");
    let type_error = broken
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == app_path && diagnostic.code == Some(2322))
        .unwrap_or_else(|| {
            panic!(
                "broken patch did not report App.vue TS2322: {:#?}",
                broken.diagnostics
            )
        });
    assert_eq!((type_error.line, type_error.column), (1, 6));
    assert!(
        type_error
            .message
            .contains("not assignable to type 'number'"),
        "unexpected TS2322 message: {}",
        type_error.message
    );
    assert_eq!(
        checker.incremental_metrics(),
        crate::batch::IncrementalCheckMetrics {
            checks: 1,
            session_starts: 1,
            last_session_started: true,
            last_requested_files: 1,
            ..Default::default()
        },
        "the first incremental check should expose its cold session work"
    );

    std::fs::write(&app_path, clean_source).expect("repair patch should write");
    let repaired = checker
        .check_incremental(std::slice::from_ref(&app_path))
        .expect("repaired incremental check should complete");
    assert!(
        repaired
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322)),
        "repaired source retained stale TS2322: {:#?}",
        repaired.diagnostics
    );
    assert_eq!(
        checker.incremental_metrics(),
        crate::batch::IncrementalCheckMetrics {
            checks: 2,
            session_starts: 1,
            session_reuses: 1,
            session_refreshes: 1,
            last_session_reused: true,
            last_session_refreshed: true,
            last_requested_files: 1,
            last_changed_files: 1,
            ..Default::default()
        },
        "the repair should expose one refreshed-file request on the reused session"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn refreshes_dependent_vue_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let app_source = r#"<script setup lang="ts">
import type { Total } from './contract'
const total: Total = 1
</script>

<template><p>{{ total }}</p></template>
"#;
    let project_root = create_project_case(
        "incremental-dependent-patch",
        &[
            ("src/App.vue", app_source),
            ("src/contract.ts", "export type Total = number\n"),
        ],
    );
    let app_path = project_root.join("src/App.vue");
    let contract_path = project_root.join("src/contract.ts");
    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker.scan_project().expect("initial scan should succeed");

    let clean = checker.check_project().expect("clean check should succeed");
    assert!(
        clean
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322)),
        "clean dependency unexpectedly reported TS2322: {:#?}",
        clean.diagnostics
    );

    std::fs::write(&contract_path, "export type Total = string\n")
        .expect("broken dependency patch should write");
    let broken = checker
        .check_incremental(std::slice::from_ref(&contract_path))
        .expect("dependent incremental check should complete");
    let dependent_error = broken
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == app_path && diagnostic.code == Some(2322))
        .unwrap_or_else(|| {
            panic!(
                "dependency patch did not refresh App.vue TS2322: {:#?}",
                broken.diagnostics
            )
        });
    assert_eq!((dependent_error.line, dependent_error.column), (2, 6));
    assert!(
        dependent_error
            .message
            .contains("not assignable to type 'string'"),
        "unexpected dependent TS2322 message: {}",
        dependent_error.message
    );

    std::fs::write(&contract_path, "export type Total = number\n")
        .expect("dependency repair should write");
    let repaired = checker
        .check_incremental(std::slice::from_ref(&contract_path))
        .expect("repaired dependency check should complete");
    assert!(
        repaired
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322)),
        "dependency repair retained stale TS2322: {:#?}",
        repaired.diagnostics
    );
    assert_eq!(
        checker.incremental_metrics(),
        crate::batch::IncrementalCheckMetrics {
            checks: 2,
            session_starts: 1,
            session_reuses: 1,
            session_refreshes: 1,
            last_session_reused: true,
            last_session_refreshed: true,
            last_requested_files: 2,
            last_changed_files: 1,
            ..Default::default()
        },
        "dependency repair should request both diagnostic inputs after one-file refresh"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn preserves_explicit_scan_scope() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let clean_source = r#"<script setup lang="ts">
const included: number = 1
</script>
"#;
    let project_root = create_project_case(
        "incremental-explicit-scope",
        &[
            ("src/Included.vue", clean_source),
            (
                "src/OutsideScope.vue",
                r#"<script setup lang="ts">
const outside: number = 'must stay outside the scan'
</script>
"#,
            ),
        ],
    );
    let included_path = project_root.join("src/Included.vue");
    let outside_path = project_root.join("src/OutsideScope.vue");
    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker
        .scan_project()
        .expect("project-wide scan should succeed");
    checker
        .scan_paths(std::slice::from_ref(&included_path))
        .expect("explicit scan should succeed");

    std::fs::write(&included_path, clean_source.replace("= 1", "= 'broken'"))
        .expect("included patch should write");
    let broken = checker
        .check_incremental(std::slice::from_ref(&included_path))
        .expect("scoped incremental check should complete");
    assert!(
        broken
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == included_path && diagnostic.code == Some(2322)),
        "included patch did not report TS2322: {:#?}",
        broken.diagnostics
    );
    assert!(
        broken
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != outside_path),
        "incremental refresh expanded the explicit scan scope: {:#?}",
        broken.diagnostics
    );

    let outside_change = checker
        .check_incremental(std::slice::from_ref(&outside_path))
        .expect("out-of-scope incremental check should complete");
    assert!(
        outside_change
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != outside_path),
        "a changed path expanded the explicit scan scope: {:#?}",
        outside_change.diagnostics
    );
    assert!(
        outside_change
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == included_path && diagnostic.code == Some(2322)),
        "out-of-scope refresh dropped the included diagnostic: {:#?}",
        outside_change.diagnostics
    );
    assert_eq!(
        checker.incremental_metrics(),
        crate::batch::IncrementalCheckMetrics {
            checks: 2,
            session_starts: 1,
            session_reuses: 1,
            last_session_reused: true,
            last_requested_files: 1,
            ..Default::default()
        },
        "an unchanged materialized scope should expose reuse without refresh work"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

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
