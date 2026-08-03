use super::{BatchTypeChecker, TypeChecker, create_project_case, resolve_test_tsgo_binary};

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
        checker.incremental_metrics(),
        crate::batch::IncrementalCheckMetrics {
            checks: 3,
            session_starts: 1,
            session_reuses: 2,
            session_refreshes: 2,
            last_session_reused: true,
            last_session_refreshed: true,
            last_requested_files: 1,
            last_changed_files: 1,
            last_deleted_files: 1,
            ..Default::default()
        },
        "the final delete should expose the deleted file and changed project config"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
