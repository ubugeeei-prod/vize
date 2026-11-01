//! Genuine native answers; held snapshots are not claims about entered IPC.

use super::*;

#[test]
#[ignore = "private two-context feasibility; exact native 7.1 Actions only"]
fn native_two_config_shared_owner_preserves_sibling_and_held_revision() {
    let native = runtime(
        "7.1.0-beta",
        "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    );
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let a = config(&root, "a", true, A_BAD);
    let b = config(&root, "b", false, B_GOOD);
    let a_uri = crate::file_uri::path_to_file_uri(&a.parent().unwrap().join("source.ts"));
    let b_uri = crate::file_uri::path_to_file_uri(&b.parent().unwrap().join("source.ts"));
    assert!(union(Some((&a_uri, A_BAD)), Some((&a_uri, A_GOOD))).is_err());
    assert!(union(Some((&a_uri, A_BAD)), Some((&a_uri, A_BAD))).is_err());
    let mut editor = EditorLspSession::spawn_with_startup_configuration(
        native.to_str().unwrap(),
        &root,
        &root,
        &a,
    )
    .unwrap();
    let first_owner = only_owner(&root, &native);
    let api = editor.configured_api.as_ref().unwrap().client.clone();
    assert_eq!(api.dialect(), Some(ApiDialect::DerivedSnapshots));
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_BAD)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let a_session = open(&api, &a, &a_uri);
    let b_session = open(&api, &b, &b_uri);
    record(
        "initial-projects",
        &json!({
            "a":a_session.project(),"b":b_session.project(),
            "aSnapshot":a_session.snapshot().handle,"bSnapshot":b_session.snapshot().handle,
            "dialect":api.dialect().unwrap().as_str(),
            "owner":{"pid":first_owner.pid,"birth":first_owner.birth,"executable":first_owner.executable},
            "physicalNativeChildren":1
        }),
    );
    assert_ne!(a_session.snapshot().handle, b_session.snapshot().handle);
    assert_eq!(
        held_type(
            a_session.client(),
            &a_session.snapshot_lease(),
            &a_session.project().id,
            &a_uri
        ),
        "number"
    );
    assert_eq!(
        held_type(
            b_session.client(),
            &b_session.snapshot_lease(),
            &b_session.project().id,
            &b_uri
        ),
        "string"
    );
    let a_bad = packet(&mut editor, "a-error", &a_uri, &a, 7);
    let b_before = packet(&mut editor, "b-before-a-repair", &b_uri, &b, 6);
    assert_diagnostics(
        &a_bad,
        expected_error(19, "Type 'string' is not assignable to type 'number'."),
    );
    assert_diagnostics(&b_before, clean());
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_NULL)), Some((&b_uri, B_NULL))).unwrap(),
    );
    let a_null = open(&api, &a, &a_uri);
    let b_null = open(&api, &b, &b_uri);
    assert_diagnostics(
        &packet(&mut editor, "a-strict-null-error", &a_uri, &a, 7),
        expected_error(19, "Type 'null' is not assignable to type 'number'."),
    );
    assert_diagnostics(
        &packet(&mut editor, "b-nonstrict-null-clean", &b_uri, &b, 6),
        clean(),
    );
    drop(a_null);
    drop(b_null);
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let a_repaired = open(&api, &a, &a_uri);
    let b_after_a = packet(&mut editor, "b-after-a-repair", &b_uri, &b, 6);
    assert_unchanged_semantic_packet(&b_after_a, &b_before);
    assert_diagnostics(&packet(&mut editor, "a-repaired", &a_uri, &a, 7), clean());
    let held_api = a_repaired.client().clone();
    let held_snapshot = a_repaired.snapshot_lease();
    let held_project = a_repaired.project().id.clone();
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_NEXT)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let a_next = open(&api, &a, &a_uri);
    assert_eq!(
        held_type(
            a_next.client(),
            &a_next.snapshot_lease(),
            &a_next.project().id,
            &a_uri
        ),
        "boolean"
    );
    assert_eq!(
        held_type(&held_api, &held_snapshot, &held_project, &a_uri),
        "number"
    );
    record(
        "distinct-held-revision",
        &json!({"currentType":"boolean","heldType":"number", "currentProject":a_next.project(),"currentSnapshot":a_next.snapshot().handle,"heldProject":held_project,"heldSnapshot":held_snapshot.handle}),
    );
    drop(a_next);
    drop(a_session);
    drop(a_repaired);
    acknowledge(&mut editor, &union(None, Some((&b_uri, B_GOOD))).unwrap());
    assert_eq!(
        held_type(&held_api, &held_snapshot, &held_project, &a_uri),
        "number"
    );
    let b_after_detach = packet(&mut editor, "b-after-a-detach", &b_uri, &b, 6);
    assert_unchanged_semantic_packet(&b_after_detach, &b_before);
    assert_eq!(only_owner(&root, &native), first_owner);
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_GOOD)), Some((&b_uri, B_BAD))).unwrap(),
    );
    let a_reopened = open(&api, &a, &a_uri);
    let b_bad = packet(&mut editor, "b-error", &b_uri, &b, 6);
    assert_diagnostics(
        &b_bad,
        expected_error(18, "Type 'number' is not assignable to type 'string'."),
    );
    assert_diagnostics(
        &packet(&mut editor, "a-after-reopen", &a_uri, &a, 7),
        clean(),
    );
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let b_repaired = open(&api, &b, &b_uri);
    let b_clean = packet(&mut editor, "b-repaired", &b_uri, &b, 6);
    assert_diagnostics(&b_clean, clean());
    assert_unchanged_semantic_packet(&b_clean, &b_before);
    assert_eq!(only_owner(&root, &native), first_owner);
    let issued = completion_item(&b_clean, "toUpperCase");
    let resolved = editor
        .completion_resolve_for_shared_context_test(&b_uri, issued.clone())
        .unwrap();
    record(
        "completion-before-restart",
        &json!({"issued":issued,"resolved":resolved}),
    );
    assert!(resolved.is_some());
    drop(held_snapshot);
    drop(held_api);
    drop(a_reopened);
    drop(b_session);
    drop(b_repaired);
    block_on(api.drain_snapshot_releases()).unwrap();
    editor.shutdown().unwrap();
    control::assert_reaped(&first_owner);
    block_on(api.close()).unwrap();
    // A real replacement has a different physical owner and receives the whole current union.
    let mut replacement = EditorLspSession::spawn_with_startup_configuration(
        native.to_str().unwrap(),
        &root,
        &root,
        &a,
    )
    .unwrap();
    let second_owner = only_owner(&root, &native);
    assert_ne!(second_owner, first_owner);
    acknowledge(
        &mut replacement,
        &union(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let replacement_api = replacement.configured_api.as_ref().unwrap().client.clone();
    let stale_resolve = replacement
        .completion_resolve_for_shared_context_test(&b_uri, issued)
        .unwrap();
    record(
        "completion-stale-after-restart",
        &json!({"result":stale_resolve}),
    );
    assert!(stale_resolve.is_none());
    let replacement_a = open(&replacement_api, &a, &a_uri);
    let replacement_b = open(&replacement_api, &b, &b_uri);
    assert_diagnostics(
        &packet(&mut replacement, "a-after-restart", &a_uri, &a, 7),
        clean(),
    );
    assert_diagnostics(
        &packet(&mut replacement, "b-after-restart", &b_uri, &b, 6),
        clean(),
    );
    drop(replacement_a);
    drop(replacement_b);
    block_on(replacement_api.drain_snapshot_releases()).unwrap();
    replacement.shutdown().unwrap();
    control::assert_reaped(&second_owner);
    block_on(replacement_api.close()).unwrap();
    assert_eq!(
        fs::read(a.parent().unwrap().join("source.ts")).unwrap(),
        A_BAD.as_bytes()
    );
    assert_eq!(
        fs::read(b.parent().unwrap().join("source.ts")).unwrap(),
        B_GOOD.as_bytes()
    );
    std::io::stdout()
        .write_all(b"TWO_CONFIG_SHARED_OWNER semantic controls passed; no resource claim\n")
        .unwrap();
}

#[test]
#[ignore = "private refusal control; exact native 7.0.2 Actions only"]
fn native_two_config_shared_owner_legacy_dialect_refuses_without_closing_owner() {
    let native = runtime(
        "7.0.2",
        "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    );
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let a = config(&root, "a", true, A_BAD);
    let uri = crate::file_uri::path_to_file_uri(&a.parent().unwrap().join("source.ts"));
    let mut editor = EditorLspSession::spawn_with_startup_configuration(
        native.to_str().unwrap(),
        &root,
        &root,
        &a,
    )
    .unwrap();
    let owner = only_owner(&root, &native);
    let api = editor.configured_api.as_ref().unwrap().client.clone();
    assert_eq!(api.dialect(), Some(ApiDialect::SessionSnapshots));
    assert!(matches!(
        api.fork_snapshot_lineage(),
        Err(corsa::CorsaError::Unsupported(_))
    ));
    acknowledge(&mut editor, &union(Some((&uri, A_BAD)), None).unwrap());
    assert_diagnostics(
        &packet(&mut editor, "legacy-after-refusal", &uri, &a, 7),
        expected_error(19, "Type 'string' is not assignable to type 'number'."),
    );
    assert_eq!(only_owner(&root, &native), owner);
    editor.shutdown().unwrap();
    control::assert_reaped(&owner);
    block_on(api.close()).unwrap();
}
