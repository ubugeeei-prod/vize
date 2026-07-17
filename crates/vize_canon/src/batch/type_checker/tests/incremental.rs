use super::super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

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
        checker.executor.incremental_session_counts(),
        (1, 1, 1),
        "broken and repaired patches should reuse one refreshed Corsa session"
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
        checker.executor.incremental_session_counts(),
        (1, 1, 1),
        "dependency patches should reuse one refreshed Corsa session"
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
        checker.executor.incremental_session_counts(),
        (1, 1, 0),
        "an unchanged materialized scope should reuse without refreshing Corsa"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn retains_added_files_across_incremental_refreshes() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let clean_source = r#"<script setup lang="ts">
const initial: number = 1
</script>
"#;
    let project_root =
        create_project_case("incremental-added-file", &[("src/App.vue", clean_source)]);
    let app_path = project_root.join("src/App.vue");
    let added_path = project_root.join("src/Added.vue");
    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker.scan_project().expect("initial scan should succeed");

    std::fs::write(
        &added_path,
        r#"<script setup lang="ts">
const added: number = 'broken added file'
</script>
"#,
    )
    .expect("added file should write");
    let after_add = checker
        .check_incremental(std::slice::from_ref(&added_path))
        .expect("added-file check should complete");
    assert!(
        after_add
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == added_path && diagnostic.code == Some(2322)),
        "added file did not report TS2322: {:#?}",
        after_add.diagnostics
    );

    std::fs::write(&app_path, clean_source.replace("= 1", "= 'broken app'"))
        .expect("app patch should write");
    let after_unrelated_edit = checker
        .check_incremental(std::slice::from_ref(&app_path))
        .expect("second incremental check should complete");
    assert!(
        after_unrelated_edit
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == added_path && diagnostic.code == Some(2322)),
        "unrelated edit dropped the added file from the project: {:#?}",
        after_unrelated_edit.diagnostics
    );
    assert!(
        after_unrelated_edit
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == app_path && diagnostic.code == Some(2322)),
        "second incremental check did not observe the App.vue patch: {:#?}",
        after_unrelated_edit.diagnostics
    );

    std::fs::remove_file(&added_path).expect("added file should delete");
    let after_delete = checker
        .check_incremental(std::slice::from_ref(&added_path))
        .expect("deleted-file check should complete");
    assert!(
        after_delete
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != added_path),
        "deleted file retained stale diagnostics: {:#?}",
        after_delete.diagnostics
    );
    assert!(
        after_delete
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file == app_path && diagnostic.code == Some(2322)),
        "deleting the added file dropped the unrelated App.vue diagnostic: {:#?}",
        after_delete.diagnostics
    );
    assert_eq!(
        checker.executor.incremental_session_counts(),
        (1, 2, 2),
        "create, edit, and delete patches should reuse one refreshed Corsa session"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
