use super::{
    Arc, ModuleLinkContextError, ModuleLinkRetirement, ModuleTargetError, SnapshotRefusal,
    block_on, project, ready,
};

#[test]
fn module_link_target_original_cancel_refuses_fresh_query_sharing_same_snapshot_allocation() {
    let (_dir, _root, source, _state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let (query, cancel) = project.begin_query(&source).unwrap();
    let original_snapshot = Arc::clone(query.snapshot());
    let a = block_on(query.run(|_| async { 7 })).unwrap();
    let checked = a
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    let (query_b, _) = project.begin_query(&source).unwrap();
    assert!(Arc::ptr_eq(&original_snapshot, query_b.snapshot()));
    let b = block_on(query_b.run(|_| async { 9 })).unwrap();
    cancel.abort();
    let mut published = false;
    assert!(matches!(
        b.publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::Source(SnapshotRefusal::Cancelled))
    ));
    assert!(!published);
}

#[test]
fn module_link_target_context_root_checker_aba_and_sticky_retirement_refuse_whole_batch() {
    for case in 0..4 {
        let (_dir, root, source, state, project) = project();
        let context = project.capture_module_link_context().unwrap();
        let result = ready(&project, &source);
        let checked = result
            .observe_module_targets(&context, &["./child.ts"])
            .unwrap()
            .recheck()
            .unwrap();
        let expected = match case {
            0 => {
                state.set_workspace_root(root.join("B"));
                state.set_workspace_root(root.clone());
                ModuleLinkContextError::Superseded
            }
            1 => {
                std::fs::write(
                    root.join("vize.config.json"),
                    "{\"typeChecker\":{\"strict\":true}}",
                )
                .unwrap();
                state.load_lsp_config(&root);
                state.load_lsp_config(&root);
                ModuleLinkContextError::Superseded
            }
            2 => {
                state.retire_module_links(ModuleLinkRetirement::InputEof);
                ModuleLinkContextError::Retired(ModuleLinkRetirement::InputEof)
            }
            _ => {
                state.retire_module_links(ModuleLinkRetirement::Shutdown);
                ModuleLinkContextError::Retired(ModuleLinkRetirement::Shutdown)
            }
        };
        let mut published = false;
        assert!(
            matches!(result.publish_with_module_targets(checked, |_, _| published = true), Err(ModuleTargetError::Context(actual)) if actual == expected)
        );
        assert!(!published);
    }
}

#[test]
fn module_link_target_concurrent_event_writer_preserves_complete_publication_envelope() {
    let (_dir, _root, source, state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let checked = result
        .observe_module_targets(&context, &["./child.ts", "./second.vue"])
        .unwrap()
        .recheck()
        .unwrap();
    let (start, started) = std::sync::mpsc::channel();
    let (attempt, attempted) = std::sync::mpsc::channel();
    let (finish, finished) = std::sync::mpsc::channel();
    let writer_state = Arc::clone(&state);
    let writer = std::thread::spawn(move || {
        started.recv().unwrap();
        attempt.send(()).unwrap();
        writer_state.observe_module_target_file_events(true);
        finish.send(()).unwrap();
    });
    assert_eq!(
        result
            .publish_with_module_targets(checked, |values, uris| {
                start.send(()).unwrap();
                attempted.recv().unwrap();
                // Rendezvous proves concurrent writer intent, not a scheduler-visible
                // lock attempt. The separate private-gate law checks the actual guard.
                assert!(matches!(
                    finished.try_recv(),
                    Err(std::sync::mpsc::TryRecvError::Empty)
                ));
                (values.0, uris.len())
            })
            .unwrap(),
        (17, 2)
    );
    writer.join().unwrap();
    finished.recv().unwrap();
    assert_eq!(
        state.with_current_module_link_context(&context, || 7),
        Ok(7)
    );
}

#[test]
fn module_link_target_caught_publication_unwind_drops_original_source_and_event_guards() {
    let (_dir, _root, source, state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let checked = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = result
            .publish_with_module_targets(checked, |_, _| panic!("authentic callback interruption"));
    }));
    assert!(interrupted.is_err());
    // Both actual writers complete after unwind; no result or stamped prefix
    // from the interrupted callback is available to a second publication.
    state.observe_module_target_file_events(true);
    project.close(&source);
    project.open(
        source.clone(),
        "/*😀*/import child from './child.ts';".into(),
        17,
        "typescript".into(),
    );
    let current = ready(&project, &source);
    let fresh = current
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    assert_eq!(
        current
            .publish_with_module_targets(fresh, |value, uris| (value.0, uris.len()))
            .unwrap(),
        (17, 1)
    );
}

#[test]
fn module_link_target_recheck_is_point_in_time_not_future_filesystem_atomicity() {
    let (_dir, root, source, _state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let checked = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    std::fs::remove_file(root.join("child.ts")).unwrap();
    assert_eq!(
        result
            .publish_with_module_targets(checked, |_, uris| uris.to_vec())
            .unwrap(),
        vec![tower_lsp::lsp_types::Url::from_file_path(root.join("child.ts")).unwrap()]
    );
}
