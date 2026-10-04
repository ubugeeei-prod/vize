//! Real original notifications precede legacy filters and await boundaries.
#![expect(
    clippy::disallowed_types,
    reason = "the real session retains its original Arc server owner"
)]
use super::{ModuleTargetError, PhysicalTargets, root};
use crate::{
    runtime::block_on,
    server::{MaestroServer, ServerState, build_lsp_service},
    source_project::{ProjectQueryResult, SourceQueryProject},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::Service;
use tower_lsp::{ClientSocket, LspService, jsonrpc::Request, lsp_types::Url};

pub(super) struct Session {
    pub(super) _dir: tempfile::TempDir,
    pub(super) root: std::path::PathBuf,
    pub(super) source: Url,
    pub(super) state: Arc<ServerState>,
    pub(super) project: SourceQueryProject<'static>,
    pub(super) service: LspService<MaestroServer>,
    pub(super) socket: ClientSocket,
}
pub(super) fn call(service: &mut LspService<MaestroServer>, input: Value) -> Option<Value> {
    let request: Request = serde_json::from_value(input).unwrap();
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service
            .call(request)
            .await
            .unwrap()
            .map(|response| serde_json::to_value(response).unwrap())
    })
}
pub(super) fn session() -> Session {
    let (dir, root, source) = root();
    std::fs::write(root.join("child.ts"), "original physical target").unwrap();
    let (mut service, socket) = build_lsp_service();
    assert_eq!(
        call(
            &mut service,
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"enabled":false}}})
        ),
        Some(
            json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":{"openClose":true,"change":2,"willSave":false,"willSaveWaitUntil":false,"save":{"includeText":false}},"workspace":{"workspaceFolders":{"supported":true,"changeNotifications":true}},"experimental":{"vize":{"jsxTypecheck":false}}},"serverInfo":{"name":"vize-maestro","version":env!("CARGO_PKG_VERSION")}}})
        )
    );
    let state = Arc::clone(&service.inner().state);
    state.set_workspace_root(root.clone());
    let project = SourceQueryProject::new_server(Arc::clone(&state));
    project.open(
        source.clone(),
        "const value=1;value;".into(),
        7,
        "javascript".into(),
    );
    Session {
        _dir: dir,
        root,
        source,
        state,
        project,
        service,
        socket,
    }
}
pub(super) fn ready(session: &Session) -> ProjectQueryResult<'static, u8> {
    let (query, _) = session.project.begin_query(&session.source).unwrap();
    block_on(query.run(|_| async { 7 })).unwrap()
}

#[test]
fn module_link_target_actual_nonempty_notifications_invalidate_before_legacy_filters() {
    for event in 0..6 {
        let mut session = session();
        let context = session.project.capture_module_link_context().unwrap();
        let result = ready(&session);
        let checked = result
            .observe_module_targets(&context, &["./child.ts"])
            .unwrap()
            .recheck()
            .unwrap();
        let input = match event {
            0 => {
                json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":"file:///node_modules/.vize/corsa-overlay/internal.ts","type":2}]}})
            }
            1 => {
                json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":"untitled:unknown","type":999}]}})
            }
            2 => {
                json!({"jsonrpc":"2.0","method":"workspace/didCreateFiles","params":{"files":[{"uri":"not-a-uri"}]}})
            }
            3 => {
                json!({"jsonrpc":"2.0","method":"workspace/didDeleteFiles","params":{"files":[{"uri":"not-a-uri"}]}})
            }
            4 => {
                json!({"jsonrpc":"2.0","method":"workspace/didRenameFiles","params":{"files":[{"oldUri":"not-old-uri","newUri":"not-new-uri"}]}})
            }
            _ => {
                json!({"jsonrpc":"2.0","method":"workspace/didRenameFiles","params":{"files":[{"oldUri":Url::from_file_path(session.root.join("folder")).unwrap(),"newUri":Url::from_file_path(session.root.join("renamed-folder")).unwrap()}]}})
            }
        };
        assert_eq!(call(&mut session.service, input.clone()), None);
        assert_eq!(call(&mut session.service, input), None);
        let mut published = false;
        assert!(matches!(
            result.publish_with_module_targets(checked, |_, _| published = true),
            Err(ModuleTargetError::EventSuperseded)
        ));
        assert!(!published);
        assert_eq!(
            session
                .state
                .with_current_module_link_context(&context, || 9),
            Ok(9)
        );
        assert_eq!(
            session.state.documents.text(&session.source).as_deref(),
            Some("const value=1;value;")
        );
    }
}

#[test]
fn module_link_target_actual_empty_notification_vectors_preserve_entire_point_result() {
    let mut session = session();
    let context = session.project.capture_module_link_context().unwrap();
    let result = ready(&session);
    let checked = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    for method in [
        "workspace/didChangeWatchedFiles",
        "workspace/didCreateFiles",
        "workspace/didDeleteFiles",
        "workspace/didRenameFiles",
    ] {
        let params = if method == "workspace/didChangeWatchedFiles" {
            json!({"changes":[]})
        } else {
            json!({"files":[]})
        };
        assert_eq!(
            call(
                &mut session.service,
                json!({"jsonrpc":"2.0","method":method,"params":params})
            ),
            None
        );
    }
    assert_eq!(
        result
            .publish_with_module_targets(checked, |value, uris| (value, uris.to_vec()))
            .unwrap(),
        (
            7,
            vec![Url::from_file_path(session.root.join("child.ts")).unwrap()]
        )
    );
}

#[test]
fn module_link_target_event_precheck_refuses_before_now_missing_physical_target() {
    let mut session = session();
    let context = session.project.capture_module_link_context().unwrap();
    let result = ready(&session);
    let observed = result
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap();
    std::fs::remove_file(session.root.join("child.ts")).unwrap();
    assert_eq!(
        call(
            &mut session.service,
            json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":Url::from_file_path(session.root.join("child.ts")).unwrap(),"type":3}]}})
        ),
        None
    );
    assert!(matches!(
        observed.recheck(),
        Err(ModuleTargetError::EventSuperseded)
    ));
    assert!(PhysicalTargets::observe(&session.root, &session.source, &["./child.ts"]).is_err());
}
