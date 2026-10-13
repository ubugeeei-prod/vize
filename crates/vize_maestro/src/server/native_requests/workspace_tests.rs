//! A workspace-wide request builds its projection outside the native
//! transaction, so other native requests are answered meanwhile.
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{FutureExt, channel::oneshot, executor::block_on};
use serde_json::{Value, json};
use tower_lsp::jsonrpc::Error;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::super::build_lsp_service;

fn reply() -> Value {
    json!({"locations": 4})
}

/// The first attempt reaches an unprepared workspace open and parks its
/// preparation; every later attempt finds the projection ready.
async fn workspace_query(
    server: &super::MaestroServer,
    attempts: &AtomicUsize,
    prepared: futures::future::Shared<oneshot::Receiver<()>>,
) -> tower_lsp::jsonrpc::Result<Value> {
    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
    let preparation = &server.state.workspace_preparation;
    if attempt == 0 {
        assert!(
            preparation.is_armed(),
            "the transaction owner arms deferral"
        );
        preparation
            .defer(Box::pin(async move {
                let _ = prepared.await;
            }))
            .await;
        // The owner abandons this attempt instead of resuming it, so a
        // resumed attempt would surface this value and fail the comparison.
        return Ok(json!("resumed an abandoned attempt"));
    }
    assert!(!preparation.is_armed(), "the second attempt never defers");
    Ok(reply())
}

#[test]
fn other_native_requests_run_while_a_workspace_projection_is_prepared() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let attempts = AtomicUsize::new(0);
    let (finish, prepared) = oneshot::channel();
    let prepared = prepared.shared();
    let mut references = Box::pin(
        server.workspace_native_request(|| workspace_query(server, &attempts, prepared.clone())),
    );
    assert!(references.as_mut().now_or_never().is_none());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(!server.state.workspace_preparation.is_armed());

    // The transaction is free while the projection is built: a hover-sized
    // native request completes at once instead of queueing behind it.
    assert_eq!(
        server
            .native_request(async { Ok(json!("hover")) })
            .now_or_never(),
        Some(Ok(json!("hover")))
    );

    finish.send(()).unwrap();
    assert_eq!(block_on(references), Ok(reply()));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn an_edit_during_preparation_refuses_the_retried_result() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let uri = Url::parse("file:///project/src/App.vue").unwrap();
    server.state.documents.open(
        uri.clone(),
        "<template>a</template>".into(),
        1,
        "vue".into(),
    );
    let attempts = AtomicUsize::new(0);
    let (finish, prepared) = oneshot::channel();
    let prepared = prepared.shared();
    let (release, held) = oneshot::channel::<()>();
    let held = held.shared();
    let mut references = Box::pin(server.workspace_native_request(|| {
        let held = held.clone();
        let query = workspace_query(server, &attempts, prepared.clone());
        async move {
            let value = query.await?;
            let _ = held.await;
            Ok(value)
        }
    }));
    assert!(references.as_mut().now_or_never().is_none());
    finish.send(()).unwrap();
    assert!(references.as_mut().now_or_never().is_none());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    server.state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<template>b</template>".into(),
        }],
        2,
    );
    release.send(()).unwrap();
    assert_eq!(block_on(references), Err(Error::content_modified()));
}

#[test]
fn a_prepared_or_cheap_request_answers_in_its_first_attempt() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let attempts = AtomicUsize::new(0);
    let result = block_on(server.workspace_native_request(|| {
        attempts.fetch_add(1, Ordering::SeqCst);
        async { Ok(reply()) }
    }));
    assert_eq!(result, Ok(reply()));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(!server.state.workspace_preparation.is_armed());
}

#[test]
fn a_caller_outside_the_owning_attempt_resumes_when_deferral_ends() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let preparation = &server.state.workspace_preparation;
    let armed = preparation.arm();
    let mut foreign = Box::pin(preparation.defer(Box::pin(async {})));
    assert!(foreign.as_mut().now_or_never().is_none());
    assert!(preparation.take().is_some());
    drop(armed);
    assert_eq!(foreign.now_or_never(), Some(()));
}
