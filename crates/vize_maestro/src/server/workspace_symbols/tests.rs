//! Cancellation is strict while pending and best effort after completion.

use std::{future::Future, task::Context};

use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request, lsp_types::InitializeResult};

use crate::{
    runtime::block_on,
    server::{MaestroServer, build_lsp_service},
};

const URI: &str = "file:///workspace/Cancellation.vue";
const SOURCE: &str = "<template><p /></template>";

fn request(value: Value) -> Request {
    serde_json::from_value(value).unwrap()
}

fn send(service: &mut LspService<MaestroServer>, value: Value) -> Option<Value> {
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service
            .call(request(value))
            .await
            .unwrap()
            .map(|response| serde_json::to_value(response).unwrap())
    })
}

fn initialized() -> LspService<MaestroServer> {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let response = send(
        &mut service,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "capabilities":{},"initializationOptions":{
                "editor":true,"lint":false,"typecheck":false,"ecosystem":false
            }
        }}),
    );
    let _: InitializeResult = serde_json::from_value(response.unwrap()["result"].clone()).unwrap();
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":URI,"languageId":"vue","version":1,"text":SOURCE}
            }})
        ),
        None
    );
    service
}

fn symbols(id: i32, query: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"workspace/symbol","params":{"query":query}})
}

fn cancel(id: i32) -> Value {
    json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}})
}

fn expected_symbols(id: i32) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":[{
        "name":"Cancellation","kind":5,
        "location":{"uri":URI,"range":{
            "start":{"line":0,"character":0},"end":{"line":0,"character":0}
        }}
    }]})
}

fn change(service: &mut LspService<MaestroServer>, version: i32) {
    let source = if version % 2 == 0 {
        "<template><span /></template>"
    } else {
        SOURCE
    };
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":URI,"version":version},"contentChanges":[{"text":source}]
            }})
        ),
        None
    );
}

#[test]
fn actual_pending_workspace_symbol_requests_cancel_during_revision_churn() {
    let mut service = initialized();
    for round in 0..9 {
        let id = 2 + round * 2;
        change(&mut service, 2 + round * 2);
        let mut pending = Box::pin(service.call(request(symbols(id, "Cancellation"))));
        // This witnesses the actual production handler being pending. Adjacent
        // stdio frames alone cannot prove that cancellation wins completion.
        assert!(
            pending
                .as_mut()
                .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                .is_pending()
        );
        assert_eq!(send(&mut service, cancel(id)), None);
        change(&mut service, 3 + round * 2);
        assert_eq!(
            serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32800,"message":"Canceled"}})
        );
        assert_eq!(
            send(&mut service, symbols(id + 1, "Cancellation")),
            Some(expected_symbols(id + 1))
        );
    }
}

#[test]
fn completed_workspace_symbol_request_ignores_late_cancel_without_poisoning_retry() {
    let mut service = initialized();
    assert_eq!(
        send(&mut service, symbols(2, "Cancellation-missing")),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
    assert_eq!(send(&mut service, cancel(2)), None);
    change(&mut service, 2);
    assert_eq!(
        send(&mut service, symbols(3, "Cancellation")),
        Some(expected_symbols(3))
    );
}
