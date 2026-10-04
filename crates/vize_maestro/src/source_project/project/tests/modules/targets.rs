//! Real ready queries own every target receipt; no native literal/link grant.
use super::{Arc, ServerState, SourceQueryProject, block_on};
use crate::{
    server::{
        ModuleLinkContextError, ModuleLinkRetirement, ModuleTargetError, WatcherCoverageError,
    },
    source_project::{ProjectQueryResult, SnapshotRefusal},
};
use std::path::PathBuf;
use tower_lsp::lsp_types::Url;
mod publication;

#[test]
fn module_link_target_borrowed_and_shared_source_hosts_refuse_before_physical_request() {
    let (_dir, _root, source, _state, server) = project();
    let context = server.capture_module_link_context().unwrap();
    let store = crate::document::DocumentStore::new();
    let borrowed = SourceQueryProject::new(&store);
    let shared = SourceQueryProject::new_shared(Arc::new(crate::document::DocumentStore::new()));
    for project in [&borrowed, &shared] {
        project.open(
            source.clone(),
            "const value=1".into(),
            17,
            "javascript".into(),
        );
        let (query, _) = project.begin_query(&source).unwrap();
        let result = block_on(query.run(|_| async { 7 })).unwrap();
        assert!(matches!(
            result.observe_module_targets(&context, &["./deliberately-missing.ts"]),
            Err(ModuleTargetError::Context(
                ModuleLinkContextError::HostUnavailable
            ))
        ));
    }
}

fn project() -> (
    tempfile::TempDir,
    PathBuf,
    Url,
    Arc<ServerState>,
    SourceQueryProject<'static>,
) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let source = Url::from_file_path(root.join("original.ts")).unwrap();
    std::fs::write(root.join("child.ts"), "physical target").unwrap();
    std::fs::write(root.join("second.vue"), "<template/>").unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.clone());
    let project = SourceQueryProject::new_server(Arc::clone(&state));
    project.open(
        source.clone(),
        "/*😀*/import child from './child.ts';".into(),
        17,
        "typescript".into(),
    );
    (dir, root, source, state, project)
}
fn ready(
    project: &SourceQueryProject<'static>,
    source: &Url,
) -> ProjectQueryResult<'static, (i32, String, String)> {
    let (query, _) = project.begin_query(source).unwrap();
    block_on(query.run(|snapshot| async move {
        (
            snapshot.version(),
            snapshot.language_id().to_owned(),
            snapshot.source().to_owned(),
        )
    }))
    .unwrap()
}

#[test]
fn module_link_target_original_host_full_batch_outputs_and_factual_unknown_coverage() {
    let (_dir, root, source, _state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let observed = result
        .observe_module_targets(&context, &["./child.ts", "./second.vue"])
        .unwrap();
    let expected = vec![
        Url::from_file_path(root.join("child.ts")).unwrap(),
        Url::from_file_path(root.join("second.vue")).unwrap(),
    ];
    assert_eq!(observed.uris(), expected);
    let checked = observed.recheck().unwrap();
    assert_eq!(
        checked.require_watcher_covered(),
        Err(WatcherCoverageError::UnknownCoverage)
    );
    assert_eq!(
        result
            .publish_with_module_targets(checked, |values, uris| (values, uris.to_vec()))
            .unwrap(),
        (
            (
                17,
                "typescript".into(),
                "/*😀*/import child from './child.ts';".into()
            ),
            expected
        )
    );
}

#[test]
fn module_link_target_source_context_event_refusal_priority_has_no_output_callback() {
    let (_dir, _root, source, state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let checked = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    project.close(&source);
    state.set_workspace_root("/changed-context".into());
    state.observe_module_target_file_events(true);
    let mut published = false;
    assert!(matches!(
        result.publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::Source(SnapshotRefusal::MissingDocument))
    ));
    assert!(!published);
}

#[test]
fn module_link_target_equal_foreign_context_and_snapshot_cannot_publish_original_batch() {
    let (_dir, root, source, _state, first) = project();
    let foreign_state = Arc::new(ServerState::new());
    foreign_state.set_workspace_root(root);
    let second = SourceQueryProject::new_server(foreign_state);
    second.open(
        source.clone(),
        "/*😀*/import child from './child.ts';".into(),
        17,
        "typescript".into(),
    );
    let original_context = first.capture_module_link_context().unwrap();
    let foreign_context = second.capture_module_link_context().unwrap();
    let a = ready(&first, &source);
    assert!(matches!(
        a.observe_module_targets(&foreign_context, &["./child.ts"]),
        Err(ModuleTargetError::Context(
            ModuleLinkContextError::ForeignSession
        ))
    ));
    let checked = a
        .observe_module_targets(&original_context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    let mut published = false;
    assert!(matches!(
        ready(&second, &source).publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::ForeignSnapshot)
    ));
    assert!(!published);
}

#[test]
fn module_link_target_late_candidate_or_final_replacement_discards_complete_prefix() {
    let (_dir, root, source, _state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    assert!(
        result
            .observe_module_targets(&context, &["./child.ts", "./missing.ts"])
            .is_err()
    );
    let observed = result
        .observe_module_targets(&context, &["./child.ts", "./second.vue"])
        .unwrap();
    std::fs::rename(root.join("second.vue"), root.join("held.vue")).unwrap();
    std::fs::write(root.join("second.vue"), "<template/>").unwrap();
    assert!(
        matches!(observed.recheck(), Err(ModuleTargetError::Changed(path)) if path == root.join("second.vue"))
    );
    assert!(matches!(
        result.observe_module_targets(&context, &[]),
        Err(ModuleTargetError::Policy(
            crate::server::TargetPolicy::EmptyBatch
        ))
    ));
}

#[test]
fn module_link_target_recheck_keeps_original_source_cancellation_and_close_reopen_laws() {
    for case in 0..5 {
        let (_dir, _root, source, _state, project) = project();
        let context = project.capture_module_link_context().unwrap();
        let (query, cancel) = project.begin_query(&source).unwrap();
        let result = block_on(query.run(|_| async { 7 })).unwrap();
        let observed = result
            .observe_module_targets(&context, &["./child.ts"])
            .unwrap();
        let expected = match case {
            0 => {
                cancel.abort();
                SnapshotRefusal::Cancelled
            }
            1 => {
                project.close(&source);
                SnapshotRefusal::MissingDocument
            }
            2 => {
                project.open(source.clone(), "different".into(), 18, "typescript".into());
                SnapshotRefusal::Superseded
            }
            3 => {
                project.close(&source);
                project.open(
                    source.clone(),
                    "/*😀*/import child from './child.ts';".into(),
                    17,
                    "typescript".into(),
                );
                SnapshotRefusal::Superseded
            }
            _ => {
                project.open(
                    source.clone(),
                    "/*😀*/import child from './child.ts';".into(),
                    17,
                    "javascript".into(),
                );
                SnapshotRefusal::Superseded
            }
        };
        assert!(
            matches!(observed.recheck(), Err(ModuleTargetError::Source(actual)) if actual == expected)
        );
    }
}

#[test]
fn module_link_target_owned_move_drop_and_forget_cannot_mint_rechecked_success() {
    let (_dir, _root, source, _state, project) = project();
    let context = project.capture_module_link_context().unwrap();
    let result = ready(&project, &source);
    let moved = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap();
    drop(moved);
    let forgotten = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap();
    std::mem::forget(forgotten);
    let fresh = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    assert_eq!(
        result
            .publish_with_module_targets(fresh, |values, _| values.0)
            .unwrap(),
        17
    );
}
