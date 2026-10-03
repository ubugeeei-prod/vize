//! Whole JSON-RPC envelopes through the actual production service builder.
mod capacity;
mod jsx;
mod lifecycle;
mod vue;

use crate::{
    runtime::block_on,
    server::{MaestroServer, build_lsp_service},
};
use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request};

const URI: &str = "file:///native.ts";

fn service() -> LspService<MaestroServer> {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let response = send(
        &mut service,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false}}}),
    );
    assert_eq!(response.as_ref().unwrap()["id"], 1);
    assert!(response.as_ref().unwrap().get("result").is_some());
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    service
}

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

fn open(service: &mut LspService<MaestroServer>, source: &str, language: &str, version: i32) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":URI,"languageId":language,"version":version,"text":source}}})
        ),
        None
    );
}

fn definition(id: i32, line: u32, character: u32) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeDefinition","params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character}}})
}
fn references(id: i32, include: bool) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeReferences","params":{"textDocument":{"uri":URI},"position":{"line":1,"character":1},"context":{"includeDeclaration":include}}})
}
fn location(line: u32, start: u32, end: u32) -> Value {
    json!({"uri":URI,"range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}}})
}
fn error(id: i32, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

#[test]
fn native_definition_and_references_have_complete_utf16_response_envelopes() {
    let mut service = service();
    open(
        &mut service,
        "/*😀*/ const café=1;\r\ncafé;\r\n",
        "typescript",
        1,
    );
    assert_eq!(
        send(&mut service, definition(2, 1, 1)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,13,17)}))
    );
    assert_eq!(
        send(&mut service, references(3, true)),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[location(0,13,17),location(1,0,4)]}))
    );
    assert_eq!(
        send(&mut service, references(4, false)),
        Some(json!({"jsonrpc":"2.0","id":4,"result":[location(1,0,4)]}))
    );
    assert_eq!(
        send(&mut service, definition(5, 0, 0)),
        Some(json!({"jsonrpc":"2.0","id":5,"result":null}))
    );
    assert_eq!(
        send(&mut service, definition(6, 0, 3)),
        Some(error(6, -32602, "Invalid native UTF-16 position"))
    );
}

#[test]
fn unsupported_language_and_actual_native_refusals_have_complete_error_envelopes() {
    let mut service = service();
    open(&mut service, "const value=1;value;", "vue", 1);
    assert_eq!(
        send(&mut service, definition(2, 0, 15)),
        Some(error(2, -32010, "Native original SFC observation refused"))
    );
    open(&mut service, "const value = /x/uv; value;", "javascript", 1);
    assert_eq!(
        send(&mut service, definition(3, 0, 7)),
        Some(error(3, -32002, "Native original Program refused"))
    );
    open(
        &mut service,
        "const value:number|string=1;value;",
        "typescript",
        1,
    );
    assert_eq!(
        send(&mut service, definition(4, 0, 7)),
        Some(error(4, -32003, "Native File observation refused"))
    );
}

#[test]
fn original_primitive_keyword_annotation_has_complete_native_definition_envelope() {
    let mut service = service();
    open(&mut service, "const value:number=1;value;", "typescript", 1);
    assert_eq!(
        send(&mut service, definition(2, 0, 22)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,6,11)}))
    );
}
