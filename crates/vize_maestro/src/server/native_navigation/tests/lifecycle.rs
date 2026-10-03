use super::{Service, URI, block_on, definition, error, location, open, request, send, service};
use serde_json::json;
use std::{future::Future, task::Context};

#[test]
fn actual_change_close_reopen_and_client_version_reset_have_whole_responses() {
    let mut service = service();
    open(&mut service, "const value=1;\nvalue;", "javascript", 1);
    assert_eq!(
        send(&mut service, definition(2, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,6,11)}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"/*😀*/ const café=1;\r\ncafé;"}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(3, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":3,"result":location(0,13,17)}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(4, 1, 1)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(&mut service, "const value=1;\nvalue;", "javascript", 1);
    assert_eq!(
        send(&mut service, definition(5, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":5,"result":location(0,6,11)}))
    );
}

#[test]
fn wire_cancel_retires_pending_native_request_without_publishing_a_location() {
    let mut service = service();
    open(&mut service, "const value=1;\nvalue;", "javascript", 1);
    let mut future = Box::pin(service.call(request(definition(9, 1, 1))));
    assert!(
        future
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
    let response = block_on(future).unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(response).unwrap(),
        error(9, -32800, "Canceled")
    );
    assert_eq!(
        send(&mut service, definition(10, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":10,"result":location(0,6,11)}))
    );
}

#[test]
fn actual_host_change_cancels_pending_wire_request_before_native_summary_publication() {
    let mut service = service();
    open(&mut service, "const value=1;\nvalue;", "javascript", 1);
    let mut future = Box::pin(service.call(request(definition(9, 1, 1))));
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    open(&mut service, "const fresh=1;\nfresh;", "javascript", 1);
    assert_eq!(
        serde_json::to_value(block_on(future).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Native query cancelled")
    );
    assert_eq!(
        send(&mut service, definition(10, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":10,"result":location(0,6,11)}))
    );
}
