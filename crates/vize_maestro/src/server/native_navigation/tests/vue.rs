//! Whole envelopes query the original Vue source through the real service.
use super::{Service, block_on, error, json, request, send, service};
use std::{future::Future, task::Context};

const URI: &str = "file:///wire-original.vue";
const SOURCE: &str = "<script setup>/*😀*/ const café=1;</script>\r\n<template>{{café}}</template>";

fn open(
    service: &mut tower_lsp::LspService<crate::server::MaestroServer>,
    source: &str,
    version: i32,
) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":URI,"languageId":"vue","version":version,"text":source}}})
        ),
        None
    );
}
fn definition(id: i32) -> serde_json::Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeDefinition","params":{"textDocument":{"uri":URI},"position":{"line":1,"character":13}}})
}
fn location(line: u32, start: u32, end: u32) -> serde_json::Value {
    json!({"uri":URI,"range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}}})
}

#[test]
fn original_vue_template_definition_and_script_references_have_full_unicode_envelopes() {
    let mut service = service();
    open(&mut service, SOURCE, 1);
    assert_eq!(
        send(&mut service, definition(2)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,27,31)}))
    );
    let references = json!({"jsonrpc":"2.0","id":3,"method":"vize/nativeReferences","params":{"textDocument":{"uri":URI},"position":{"line":0,"character":28},"context":{"includeDeclaration":true}}});
    assert_eq!(
        send(&mut service, references),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[location(0,27,31),location(1,12,16)]}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":4,"method":"vize/nativeReferences","params":{"textDocument":{"uri":URI},"position":{"line":1,"character":13},"context":{"includeDeclaration":false}}})
        ),
        Some(json!({"jsonrpc":"2.0","id":4,"result":[location(1,12,16)]}))
    );
}

#[test]
fn vue_source_change_close_and_reopen_publish_only_current_whole_responses() {
    let mut service = service();
    open(&mut service, SOURCE, 1);
    assert!(
        send(&mut service, definition(2))
            .unwrap()
            .get("result")
            .is_some()
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"<script setup>const fresh=1;</script>\n<template>{{fresh}}</template>"}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(3)),
        Some(json!({"jsonrpc":"2.0","id":3,"result":location(0,20,25)}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(4)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(&mut service, SOURCE, 1);
    assert_eq!(
        send(&mut service, definition(5)),
        Some(json!({"jsonrpc":"2.0","id":5,"result":location(0,27,31)}))
    );
}

#[test]
fn actual_vue_refusal_and_cancel_have_whole_error_envelopes_and_no_legacy_location() {
    let mut service = service();
    open(
        &mut service,
        "<script>const value=1;</script>\n<template>{{value}}</template>",
        1,
    );
    assert_eq!(
        send(&mut service, definition(2)),
        Some(error(2, -32010, "Native original SFC observation refused"))
    );
    assert_eq!(
        send(&mut service, definition(3)),
        Some(error(3, -32010, "Native original SFC observation refused"))
    );
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(definition(9))));
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
    assert_eq!(
        send(&mut service, definition(10)),
        Some(json!({"jsonrpc":"2.0","id":10,"result":location(0,27,31)}))
    );
}
