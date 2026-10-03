use super::{URI, definition, error, location, open, send, service};
use crate::runtime::block_on;
use serde_json::json;
use std::{future::Future, task::Context};
use tower::Service;
use tower_lsp::jsonrpc::Request;

#[test]
fn actual_react_language_change_close_and_reopen_have_complete_guarded_responses() {
    let mut service = service();
    let source = "const UI=1;const view=<UI/>;";
    open(&mut service, source, "javascriptreact", 1);
    assert_eq!(
        send(&mut service, definition(2, 0, 23)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,6,8)}))
    );
    open(&mut service, source, "typescript", 1);
    assert_eq!(
        send(&mut service, definition(3, 0, 23)),
        Some(error(3, -32002, "Native original Program refused"))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(4, 0, 23)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(&mut service, source, "typescriptreact", 1);
    assert_eq!(
        send(&mut service, definition(5, 0, 23)),
        Some(json!({"jsonrpc":"2.0","id":5,"result":location(0,6,8)}))
    );
}

#[test]
fn actual_pending_react_requests_cancel_before_change_can_publish_original_locations() {
    let mut service = service();
    open(
        &mut service,
        "const UI=1;\nconst view=<UI/>;",
        "javascriptreact",
        1,
    );
    let request: Request = serde_json::from_value(definition(9, 1, 13)).unwrap();
    let mut pending = Box::pin(service.call(request));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":9}})
        ),
        None
    );
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Canceled")
    );
    let request: Request = serde_json::from_value(definition(10, 1, 13)).unwrap();
    let mut pending = Box::pin(service.call(request));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"const Other=1;\nconst view=<Other/>;"}]}})
        ),
        None
    );
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(10, -32800, "Native query cancelled")
    );
    assert_eq!(
        send(&mut service, definition(11, 1, 13)),
        Some(json!({"jsonrpc":"2.0","id":11,"result":location(0,6,11)}))
    );
}
