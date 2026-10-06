//! Complete notification ownership across queued edits, retries and transport.

use std::future::Future;
use tower_lsp::LanguageServer;
use tower_lsp::lsp_types::{DidSaveTextDocumentParams, TextDocumentIdentifier};

use super::*;

const CORPUS_APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp-regressions/diagnostic-publication-8012/App.vue.txt"
);
const CORPUS_DEPENDENCY: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp-regressions/diagnostic-publication-8012/useToast.ts.txt"
);
const ORIGINAL_STREAM: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp-regressions/diagnostic-publication-8012/b899-publish-stream.json"
);

pub(super) fn drain(socket: &mut ClientSocket, work: impl Future<Output = ()>) -> Vec<Value> {
    futures::executor::block_on(async {
        futures::pin_mut!(work);
        let mut messages = Vec::new();
        loop {
            futures::select! {
                () = work.as_mut().fuse() => break,
                request = socket.next().fuse() => {
                    messages.push(serde_json::to_value(request.unwrap()).unwrap());
                }
            }
        }
        while let Some(Some(request)) = socket.next().now_or_never() {
            messages.push(serde_json::to_value(request).unwrap());
        }
        messages
    })
}

pub(super) fn diagnostics(messages: Vec<Value>) -> Vec<Value> {
    messages
        .into_iter()
        .filter(|message| message["method"] == "textDocument/publishDiagnostics")
        .collect()
}

pub(super) fn notification(fixture: &Fixture, collected: &CollectedDiagnostics) -> Value {
    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":whole(fixture, collected)})
}

#[test]
fn queued_old_edits_cannot_adopt_the_latest_version_after_waiting_for_native_scope() {
    assert_eq!(CORPUS_APP, APP);
    assert_eq!(CORPUS_DEPENDENCY, TOAST);
    let original: Value = serde_json::from_str(ORIGINAL_STREAM).unwrap();
    assert_eq!(original["publishStream"].as_array().unwrap().len(), 249);
    assert_eq!(original["publishStream"][244]["version"], 87);
    assert_eq!(
        original["publishStream"][244],
        original["publishStream"][245]
    );
    assert_eq!(
        original["publishStream"][245],
        original["publishStream"][246]
    );
    let mut fixture = fixture(false);
    change(&fixture, &fixture.app, 83, CORPUS_APP);
    let server = fixture.service.inner();
    let scope = futures::executor::block_on(server.state.corsa_request_scope());
    let mut edits = Vec::new();
    for version in 84..=87 {
        let mut edit = Box::pin(server.apply_document_changes(
            &fixture.app,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: CORPUS_APP.to_owned(),
            }],
            version,
        ));
        assert!(edit.as_mut().now_or_never().is_none());
        assert_eq!(server.state.documents.version(&fixture.app), Some(version));
        edits.push(edit);
    }
    drop(scope);
    let observed = diagnostics(drain(&mut fixture.socket, async {
        futures::future::join_all(edits).await;
    }));
    let sync = crate::ide::DiagnosticService::collect(&fixture.service.inner().state, &fixture.app);
    let mut expected = (84..=87)
        .map(|version| json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":fixture.app,"version":version,"diagnostics":sync}}))
        .collect::<Vec<_>>();
    // Version identifies each authored source; only the final slot is complete.
    expected.push(notification(&fixture, &current(&fixture)));
    assert_eq!(observed, expected);
    assert_eq!(observed[4]["params"]["version"], 87);
}

#[test]
fn an_already_taken_retry_and_foreground_pass_publish_one_complete_current_result() {
    let mut fixture = fixture(false);
    let server = fixture.service.inner();
    let scope = futures::executor::block_on(server.state.corsa_request_scope());
    let mut retry = Box::pin(server.publish_diagnostics_if_version(&fixture.app, 1));
    let mut foreground = Box::pin(server.publish_diagnostics(&fixture.app));
    assert!(retry.as_mut().now_or_never().is_none());
    assert!(foreground.as_mut().now_or_never().is_none());
    drop(scope);
    let observed = diagnostics(drain(&mut fixture.socket, async {
        futures::join!(retry, foreground);
    }));
    assert_eq!(observed, vec![notification(&fixture, &current(&fixture))]);
}

#[test]
fn changed_dependency_source_republishes_the_complete_unchanged_root_version() {
    let mut fixture = fixture(false);
    let dependency = Url::from_file_path(fixture.root.path().join("useToast.ts")).unwrap();
    fixture.service.inner().state.documents.open(
        dependency.clone(),
        CORPUS_DEPENDENCY.to_owned(),
        81,
        "typescript".to_owned(),
    );
    let initial = notification(&fixture, &current(&fixture));
    let observed = diagnostics(drain(
        &mut fixture.socket,
        fixture.service.inner().publish_diagnostics(&fixture.app),
    ));
    assert_eq!(observed, vec![initial]);
    let state = &fixture.service.inner().state;
    assert!(state.documents.apply_changes(
        &dependency,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: CORPUS_DEPENDENCY.replace("text: string", "text: number"),
        }],
        82,
    ));
    state.mark_corsa_disk_state_dirty();
    let expected = notification(&fixture, &current(&fixture));
    assert_eq!(expected["params"]["version"], 1);
    let observed = diagnostics(drain(
        &mut fixture.socket,
        fixture.service.inner().publish_diagnostics(&fixture.app),
    ));
    assert_eq!(observed, vec![expected]);
}

#[test]
fn cancelling_a_backpressured_flush_does_not_retract_or_duplicate_its_queued_notification() {
    let mut fixture = fixture(false);
    let expected = notification(&fixture, &current(&fixture));
    let server = fixture.service.inner();
    // The pinned client's single-slot channel is now occupied. Its next
    // cloned sender enqueues once, then waits for the receiver in poll_flush.
    assert!(
        server
            .client
            .publish_diagnostics(fixture.unrelated.clone(), Vec::new(), None)
            .now_or_never()
            .is_some()
    );
    let mut sending = Box::pin(server.publish_diagnostics(&fixture.app));
    assert!(sending.as_mut().now_or_never().is_none());
    let lock = server.state.diagnostic_lock(&fixture.app);
    assert!(lock.lock().now_or_never().is_some());
    drop(sending);
    let observed = drain(
        &mut fixture.socket,
        server.publish_diagnostics_if_version(&fixture.app, 1),
    );
    let filler = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":fixture.unrelated,"diagnostics":[]}});
    let notice = json!({"jsonrpc":"2.0","method":"window/showMessage","params":{"type":2,"message":crate::ide::diagnostics::TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE}});
    assert_eq!(observed, vec![filler, expected, notice]);
}

#[test]
fn a_stale_claim_cannot_replace_a_newer_complete_publication_owner() {
    let mut fixture = fixture(false);
    let stale = current(&fixture);
    change(
        &fixture,
        &fixture.unrelated,
        2,
        "<template>new source world</template>\n",
    );
    let newer = current(&fixture);
    let expected = notification(&fixture, &newer);
    let observed = diagnostics(drain(
        &mut fixture.socket,
        fixture
            .service
            .inner()
            .publish_collected_diagnostics(&fixture.app, newer),
    ));
    assert_eq!(observed, vec![expected]);
    let state = &fixture.service.inner().state;
    let document = state.diagnostic_lock(&fixture.app);
    assert!(
        document
            .claim(
                state,
                &fixture.app,
                stale.version,
                stale.stamp.unwrap(),
                &stale.diagnostics,
                false
            )
            .is_err()
    );
    let observed = diagnostics(drain(
        &mut fixture.socket,
        fixture.service.inner().publish_diagnostics(&fixture.app),
    ));
    assert_eq!(observed, Vec::<Value>::new());
    // The still-retained older identity must also refuse, rather than taking
    // the duplicate branch after the current document version moves.
    let expired = current(&fixture);
    change(&fixture, &fixture.app, 2, APP);
    assert!(
        document
            .claim(
                state,
                &fixture.app,
                expired.version,
                expired.stamp.unwrap(),
                &expired.diagnostics,
                false
            )
            .is_err()
    );
    let expected = notification(&fixture, &current(&fixture));
    let observed = diagnostics(drain(
        &mut fixture.socket,
        fixture
            .service
            .inner()
            .publish_diagnostics_if_version(&fixture.app, 2),
    ));
    assert_eq!(observed, vec![expected]);
}

#[test]
fn explicit_saves_keep_each_complete_notification_at_the_same_source_and_version() {
    let mut fixture = fixture(false);
    let expected = notification(&fixture, &current(&fixture));
    for _ in 0..2 {
        let observed = diagnostics(drain(
            &mut fixture.socket,
            fixture.service.inner().did_save(DidSaveTextDocumentParams {
                text_document: TextDocumentIdentifier {
                    uri: fixture.app.clone(),
                },
                text: None,
            }),
        ));
        assert_eq!(observed, vec![expected.clone()]);
    }
    assert_eq!(
        fixture
            .service
            .inner()
            .state
            .documents
            .version(&fixture.app),
        Some(1)
    );
}
