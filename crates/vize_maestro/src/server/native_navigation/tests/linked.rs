//! Complete standard RPC envelopes under an explicit native initialization key.
use super::{Service, block_on, error, request, send};
use crate::server::{MaestroServer, build_lsp_service};
use serde_json::{Value, json};
use std::{future::Future, task::Context};
use tower_lsp::LspService;
const URI: &str = "file:///App.vue";
const SOURCE: &str = include_str!("../../../../tests/fixtures/native-linked-history-3471.vue");
fn service(native: bool, rename: bool) -> LspService<MaestroServer> {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let initialized=send(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false,"nativeLinkedEditing":native,"rename":rename}}})).unwrap();
    assert_eq!(
        initialized["result"]["capabilities"]["linkedEditingRangeProvider"],
        if rename { json!(true) } else { Value::Null }
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    service
}
fn open(service: &mut LspService<MaestroServer>, source: &str, version: i32) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":URI,"languageId":"vue","version":version,"text":source}}})
        ),
        None
    );
}
fn linked(id: i32, line: u32, character: u32) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"textDocument/linkedEditingRange","params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character}}})
}
fn pair(id: i32, first: (u32, u32, u32), second: (u32, u32, u32)) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":{"ranges":[
        {"start":{"line":first.0,"character":first.1},"end":{"line":first.0,"character":first.2}},
        {"start":{"line":second.0,"character":second.1},"end":{"line":second.0,"character":second.2}}
    ]}})
}
#[test]
fn original_3471_standard_inner_pair_responses_are_complete_and_dev_oracle_equal() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    for (id, line, character, first, second) in [
        (2, 6, 6, (6, 5, 8), (6, 32, 35)),
        (3, 6, 33, (6, 5, 8), (6, 32, 35)),
        (4, 6, 8, (6, 5, 8), (6, 32, 35)),
        (5, 5, 4, (5, 3, 6), (7, 4, 7)),
    ] {
        let expected = pair(id, first, second);
        let offset = crate::ide::position_to_offset(SOURCE, line, character).unwrap();
        let oracle =
            crate::ide::linked_editing::LinkedEditingService::ranges(SOURCE, "/App.vue", offset)
                .unwrap();
        assert_eq!(serde_json::to_value(oracle).unwrap(), expected["result"]);
        assert_eq!(
            send(&mut service, linked(id, line, character)),
            Some(expected)
        );
    }
    use sha2::{Digest, Sha256};
    let digest: [u8; 32] = Sha256::digest(SOURCE.as_bytes()).into();
    assert_eq!(
        digest,
        [
            0xa6, 0xb9, 0xb9, 0x8e, 0xde, 0x93, 0xce, 0x6a, 0x05, 0x92, 0x5d, 0xab, 0x00, 0x2d,
            0xb4, 0x81, 0x52, 0x94, 0x94, 0x41, 0x49, 0x9a, 0x8e, 0x36, 0x2b, 0xef, 0xe2, 0x07,
            0x8d, 0xe3, 0xb1, 0x35
        ]
    );
}
#[test]
fn explicit_opt_in_keeps_outer_frame_unsupported_and_default_route_unchanged() {
    let mut native = service(true, true);
    open(&mut native, SOURCE, 1);
    assert_eq!(
        send(&mut native, linked(2, 4, 3)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
    let mut legacy = service(false, true);
    open(&mut legacy, SOURCE, 1);
    assert_eq!(
        send(&mut legacy, linked(2, 4, 3)),
        Some(pair(2, (4, 1, 9), (9, 2, 10)))
    );
    let mut disabled = service(true, false);
    open(&mut disabled, SOURCE, 1);
    assert_eq!(
        send(&mut disabled, linked(2, 6, 6)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
}
#[test]
fn native_non_names_singles_and_case_mismatch_have_whole_null_responses() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    for (id, line, character) in [(2, 6, 4), (3, 6, 10), (4, 6, 25), (5, 1, 8), (6, 8, 4)] {
        assert_eq!(
            send(&mut service, linked(id, line, character)),
            Some(json!({"jsonrpc":"2.0","id":id,"result":null}))
        );
    }
    open(&mut service, "<template><DIV></div></template>", 2);
    assert_eq!(
        send(&mut service, linked(7, 0, 12)),
        Some(json!({"jsonrpc":"2.0","id":7,"result":null}))
    );
}
#[test]
fn native_utf16_invalid_positions_and_recovered_sources_keep_whole_error_responses() {
    let mut service = service(true, true);
    open(&mut service, "<template>😀<Card-é></Card-é></template>", 1);
    assert_eq!(
        send(&mut service, linked(2, 0, 14)),
        Some(pair(2, (0, 13, 19), (0, 22, 28)))
    );
    assert_eq!(
        send(&mut service, linked(3, 0, 11)),
        Some(error(3, -32602, "Invalid native UTF-16 position"))
    );
    open(
        &mut service,
        "<template><p></p><div title='unterminated</template>",
        2,
    );
    assert_eq!(
        send(&mut service, linked(4, 0, 11)),
        Some(error(4, -32012, "Native original template names refused"))
    );
}
#[test]
fn standard_native_change_close_and_reopen_have_complete_response_envelopes() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    assert_eq!(
        send(&mut service, linked(2, 6, 6)),
        Some(pair(2, (6, 5, 8), (6, 32, 35)))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"<template><span></span></template>"}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, linked(3, 0, 12)),
        Some(pair(3, (0, 11, 15), (0, 18, 22)))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, linked(4, 0, 12)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(&mut service, SOURCE, 1);
    assert_eq!(
        send(&mut service, linked(5, 6, 6)),
        Some(pair(5, (6, 5, 8), (6, 32, 35)))
    );
}
#[test]
fn standard_native_wire_cancel_returns_complete_cancelled_envelope_without_pair() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
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
        send(&mut service, linked(10, 6, 6)),
        Some(pair(10, (6, 5, 8), (6, 32, 35)))
    );
}
#[test]
fn standard_native_real_host_change_cancels_pending_wire_pair_publication() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    open(&mut service, "<template><span></span></template>", 1);
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Native query cancelled")
    );
    assert_eq!(
        send(&mut service, linked(10, 0, 12)),
        Some(pair(10, (0, 11, 15), (0, 18, 22)))
    );
}

#[test]
fn exact_initialization_opt_in_disable_refuses_in_flight_standard_publication() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":false})));
    let response = serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap();
    assert_eq!(
        response,
        json!({"jsonrpc":"2.0","id":9,"error":{"code":-32801,"message":"Content modified"}})
    );
    // The next request follows the preserved default route.
    assert_eq!(
        send(&mut service, linked(10, 4, 3)),
        Some(pair(10, (4, 1, 9), (9, 2, 10)))
    );
}
