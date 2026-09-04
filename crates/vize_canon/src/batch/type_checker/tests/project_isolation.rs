#![cfg(unix)]

use super::{BatchTypeChecker, DeclarationEmitOptions, resolve_test_tsgo_binary, unique_case_dir};
use crate::batch::TypeChecker;
use std::{
    sync::{Arc, Barrier},
    thread,
};

mod support;

use support::*;

#[test]
fn concurrent_projects_share_dependencies_without_sharing_mutable_state() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let case_root = unique_case_dir("shared-node-modules-concurrent-projects");
    let _ = std::fs::remove_dir_all(&case_root);
    let first_root = case_root.join("first");
    let second_root = case_root.join("second");
    let shared_node_modules = case_root.join("node_modules");
    populate_shared_dependency_tree(&shared_node_modules);
    create_project(&first_root, "first");
    create_project(&second_root, "second");
    link_shared_node_modules(&first_root, &shared_node_modules);
    link_shared_node_modules(&second_root, &shared_node_modules);

    let first_app = first_root.join("src/App.vue");
    let first_clean_source = std::fs::read_to_string(&first_app).unwrap();

    let first = scanned_checker(&first_root);
    let second = scanned_checker(&second_root);
    assert_canonical_virtual_paths(&first, &first_root, &shared_node_modules);
    assert_canonical_virtual_paths(&second, &second_root, &shared_node_modules);

    let (first_clean, second_clean) = concurrent_checks(&first, &second);
    assert_clean_and_owned(&first_clean, &first_root, &second_root, "first clean check");
    assert_clean_and_owned(
        &second_clean,
        &second_root,
        &first_root,
        "second clean check",
    );
    assert_materialized_source(&first, "const checked: 'first' = owner");
    assert_materialized_source(&second, "const checked: 'second' = owner");

    std::fs::write(&first_app, first_clean_source.replace("= owner", "= 0")).unwrap();
    let first_broken_checker = scanned_checker(&first_root);
    let second_unchanged_checker = scanned_checker(&second_root);
    let (first_broken, second_unchanged) =
        concurrent_checks(&first_broken_checker, &second_unchanged_checker);
    let type_error = first_broken
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == first_app && diagnostic.code == Some(2322))
        .unwrap_or_else(|| {
            panic!(
                "broken first project did not report its authored TS2322: {:#?}",
                first_broken.diagnostics
            )
        });
    assert_eq!((type_error.line, type_error.column), (2, 6));
    assert_clean_and_owned(
        &second_unchanged,
        &second_root,
        &first_root,
        "second check while first is broken",
    );

    std::fs::write(&first_app, &first_clean_source).unwrap();
    let first_repaired_checker = scanned_checker(&first_root);
    let second_reused_checker = scanned_checker(&second_root);
    let (first_repaired, second_reused) =
        concurrent_checks(&first_repaired_checker, &second_reused_checker);
    assert_clean_and_owned(
        &first_repaired,
        &first_root,
        &second_root,
        "first repaired check",
    );
    assert_clean_and_owned(
        &second_reused,
        &second_root,
        &first_root,
        "second reused check",
    );
    assert_materialized_source(&first_repaired_checker, "const checked: 'first' = owner");
    assert_materialized_source(&second_reused_checker, "const checked: 'second' = owner");

    let (first_declarations, second_declarations) = thread::scope(|scope| {
        let first_checker = &first_repaired_checker;
        let second_checker = &second_reused_checker;
        let barrier = Arc::new(Barrier::new(2));
        let first_barrier = Arc::clone(&barrier);
        let first_out = first_root.join("types");
        let second_out = second_root.join("types");
        let first_task = scope.spawn(move || {
            first_barrier.wait();
            first_checker.emit_declarations(&DeclarationEmitOptions::new(first_out))
        });
        let second_task = scope.spawn(move || {
            barrier.wait();
            second_checker.emit_declarations(&DeclarationEmitOptions::new(second_out))
        });
        (
            first_task
                .join()
                .unwrap()
                .expect("first emit should succeed"),
            second_task
                .join()
                .unwrap()
                .expect("second emit should succeed"),
        )
    });
    assert_declaration_owner(first_declarations.files, "first", "second");
    assert_declaration_owner(second_declarations.files, "second", "first");
    assert_materialized_source(&first_repaired_checker, "const checked: 'first' = owner");
    assert_materialized_source(&second_reused_checker, "const checked: 'second' = owner");

    let _ = std::fs::remove_dir_all(&case_root);
}

#[test]
fn persistent_sessions_refresh_independently_over_shared_dependencies() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let case_root = unique_case_dir("shared-node-modules-persistent-sessions");
    let _ = std::fs::remove_dir_all(&case_root);
    let first_root = case_root.join("first");
    let second_root = case_root.join("second");
    let shared_node_modules = case_root.join("node_modules");
    populate_shared_dependency_tree(&shared_node_modules);
    create_relative_project(&first_root, "first");
    create_relative_project(&second_root, "second");
    link_shared_node_modules(&first_root, &shared_node_modules);
    link_shared_node_modules(&second_root, &shared_node_modules);

    let first_app = first_root.join("src/App.vue");
    let second_app = second_root.join("src/App.vue");
    let first_clean_source = std::fs::read_to_string(&first_app).unwrap();
    let second_clean_source = std::fs::read_to_string(&second_app).unwrap();
    let mut first = scanned_checker(&first_root);
    let mut second = scanned_checker(&second_root);
    assert_canonical_virtual_paths(&first, &first_root, &shared_node_modules);
    assert_canonical_virtual_paths(&second, &second_root, &shared_node_modules);

    std::fs::write(
        &first_app,
        first_clean_source.replace("= 'first'", "= 'broken-first'"),
    )
    .unwrap();
    let first_broken = first
        .check_incremental(std::slice::from_ref(&first_app))
        .expect("first persistent session should start");
    assert_type_error(&first_broken, &first_app);

    let second_clean = second
        .check_incremental(std::slice::from_ref(&second_app))
        .expect("second persistent session should start");
    assert_clean_and_owned(
        &second_clean,
        &second_root,
        &first_root,
        "second persistent session",
    );
    assert_materialized_source(&first, "const checked: Owner = 'broken-first'");
    assert_materialized_source(&second, "const checked: Owner = 'second'");

    std::fs::write(&first_app, &first_clean_source).unwrap();
    let first_repaired = first
        .check_incremental(std::slice::from_ref(&first_app))
        .expect("first persistent session should refresh");
    assert_clean_and_owned(
        &first_repaired,
        &first_root,
        &second_root,
        "first repaired persistent session",
    );

    std::fs::write(
        &second_app,
        second_clean_source.replace("= 'second'", "= 'broken-second'"),
    )
    .unwrap();
    let second_broken = second
        .check_incremental(std::slice::from_ref(&second_app))
        .expect("second persistent session should refresh");
    assert_type_error(&second_broken, &second_app);

    let first_reused = first
        .check_incremental(std::slice::from_ref(&first_app))
        .expect("first persistent session should remain isolated");
    assert_clean_and_owned(
        &first_reused,
        &first_root,
        &second_root,
        "first reused persistent session",
    );
    assert_materialized_source(&first, "const checked: Owner = 'first'");
    assert_materialized_source(&second, "const checked: Owner = 'broken-second'");

    std::fs::write(&second_app, &second_clean_source).unwrap();
    let second_repaired = second
        .check_incremental(std::slice::from_ref(&second_app))
        .expect("second persistent session should repair");
    assert_clean_and_owned(
        &second_repaired,
        &second_root,
        &first_root,
        "second repaired persistent session",
    );
    assert_eq!(first.incremental_metrics().session_starts, 1);
    assert_eq!(first.incremental_metrics().session_reuses, 2);
    assert_eq!(second.incremental_metrics().session_starts, 1);
    assert_eq!(second.incremental_metrics().session_reuses, 2);

    let _ = std::fs::remove_dir_all(&case_root);
}
