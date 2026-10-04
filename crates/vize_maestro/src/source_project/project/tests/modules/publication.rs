use super::super::super::ModuleLinkPublicationError;
use super::{Arc, ModuleLinkContextError, ServerState, SourceQueryProject, block_on, uri};
use crate::server::ModuleLinkRetirement;
use crate::source_project::{ProjectQueryResult, SnapshotRefusal};
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

fn project() -> (Arc<ServerState>, SourceQueryProject<'static>) {
    let state = Arc::new(ServerState::new());
    state.set_workspace_root("/actual-source-host".into());
    let project = SourceQueryProject::new_server(Arc::clone(&state));
    project.open(
        uri("file:///actual-source-host/original.ts"),
        "/*😀*/const café=1;\r\ncafé;".into(),
        17,
        "typescript".into(),
    );
    (state, project)
}
fn ready(
    project: &SourceQueryProject<'static>,
) -> ProjectQueryResult<'static, (i32, String, String)> {
    let (query, _) = project
        .begin_query(&uri("file:///actual-source-host/original.ts"))
        .unwrap();
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
fn module_link_publication_uses_only_its_own_host_and_preserves_complete_owned_data() {
    let (_, first) = project();
    let (_, second) = project();
    let a = first.capture_module_link_context().unwrap();
    let foreign_ready = ready(&second);
    let mut published = false;
    assert_eq!(
        foreign_ready.publish_with_module_link_context(&a, |_| published = true),
        Err(ModuleLinkPublicationError::Context(
            ModuleLinkContextError::ForeignSession
        ))
    );
    assert!(!published);
    assert_eq!(
        ready(&first).publish_with_module_link_context(&a, |values| values),
        Ok((
            17,
            "typescript".into(),
            "/*😀*/const café=1;\r\ncafé;".into()
        ))
    );
}

#[test]
fn module_link_ready_publication_refuses_real_config_root_aba_and_retirement_without_callback() {
    let (state, project) = project();
    let a = project.capture_module_link_context().unwrap();
    let result = ready(&project);
    state.set_workspace_root("/B".into());
    state.set_workspace_root("/actual-source-host".into());
    let mut published = false;
    assert_eq!(
        result.publish_with_module_link_context(&a, |_| published = true),
        Err(ModuleLinkPublicationError::Context(
            ModuleLinkContextError::Superseded
        ))
    );
    assert!(!published);
    let fresh = project.capture_module_link_context().unwrap();
    let result = ready(&project);
    state.retire_module_links(ModuleLinkRetirement::Shutdown);
    assert_eq!(
        result.publish_with_module_link_context(&fresh, |_| published = true),
        Err(ModuleLinkPublicationError::Context(
            ModuleLinkContextError::Retired(ModuleLinkRetirement::Shutdown)
        ))
    );
    assert!(!published);
    assert_eq!(
        state
            .documents
            .text(&uri("file:///actual-source-host/original.ts"))
            .as_deref(),
        Some("/*😀*/const café=1;\r\ncafé;")
    );
}

#[test]
fn module_link_source_version_language_close_reopen_and_cancel_keep_original_refusals() {
    for case in 0..6 {
        let (state, project) = project();
        let context = project.capture_module_link_context().unwrap();
        let (query, cancel) = project
            .begin_query(&uri("file:///actual-source-host/original.ts"))
            .unwrap();
        let result = block_on(query.run(|_| async { vec![7, 9] })).unwrap();
        let target = uri("file:///actual-source-host/original.ts");
        let expected = match case {
            0 => {
                assert!(project.apply_changes(
                    &target,
                    vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: "different".into()
                    }],
                    18
                ));
                SnapshotRefusal::Superseded
            }
            1 => {
                state.documents.open(
                    target,
                    "/*😀*/const café=1;\r\ncafé;".into(),
                    18,
                    "typescript".into(),
                );
                SnapshotRefusal::Superseded
            }
            2 => {
                state.documents.open(
                    target,
                    "/*😀*/const café=1;\r\ncafé;".into(),
                    17,
                    "javascript".into(),
                );
                SnapshotRefusal::Superseded
            }
            3 => {
                project.close(&target);
                SnapshotRefusal::MissingDocument
            }
            4 => {
                project.close(&target);
                project.open(
                    target,
                    "/*😀*/const café=1;\r\ncafé;".into(),
                    17,
                    "typescript".into(),
                );
                SnapshotRefusal::Superseded
            }
            _ => {
                cancel.abort();
                SnapshotRefusal::Cancelled
            }
        };
        let mut published = false;
        assert_eq!(
            result.publish_with_module_link_context(&context, |_| published = true),
            Err(ModuleLinkPublicationError::Source(expected))
        );
        assert!(!published);
    }
}

#[test]
fn module_link_actual_writer_waits_until_original_source_and_context_publication_finishes() {
    for checker in [false, true] {
        let (state, project) = project();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("vize.config.json"),
            r#"{"typeChecker":{"strict":true,"lspRequestTimeoutMs":91000}}"#,
        )
        .unwrap();
        let context = project.capture_module_link_context().unwrap();
        let result = ready(&project);
        let (start, started) = std::sync::mpsc::channel();
        let (attempt, attempted) = std::sync::mpsc::channel();
        let (finish, finished) = std::sync::mpsc::channel();
        let writer_state = Arc::clone(&state);
        let path = dir.path().to_path_buf();
        let writer = std::thread::spawn(move || {
            started.recv().unwrap();
            attempt.send(()).unwrap();
            if checker {
                writer_state.load_lsp_config(&path);
            } else {
                writer_state.set_workspace_root("/changed-root".into());
            }
            finish.send(()).unwrap();
        });
        assert_eq!(
            result.publish_with_module_link_context(&context, |values| {
                start.send(()).unwrap();
                attempted.recv().unwrap();
                assert!(matches!(
                    finished.try_recv(),
                    Err(std::sync::mpsc::TryRecvError::Empty)
                ));
                values
            }),
            Ok((
                17,
                "typescript".into(),
                "/*😀*/const café=1;\r\ncafé;".into()
            ))
        );
        writer.join().unwrap();
        finished.recv().unwrap();
        assert_eq!(
            state.with_current_module_link_context(&context, || ()),
            Err(ModuleLinkContextError::Superseded)
        );
        let fresh = project.capture_module_link_context().unwrap();
        if checker {
            assert!(fresh.checker().strict);
            assert_eq!(fresh.timeout_ms(), 91_000);
        } else {
            assert_eq!(fresh.root(), std::path::Path::new("/changed-root"));
        }
    }
}
