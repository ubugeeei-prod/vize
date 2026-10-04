use super::*;

#[test]
fn original_3471_outer_frame_whole_rpc_envelopes_are_ordered_and_dev_oracle_equal() {
    let mut native = service(true, true);
    open(&mut native, SOURCE, 1);
    for (id, line, character) in [(2, 4, 1), (3, 4, 3), (4, 4, 9), (5, 9, 2), (6, 9, 10)] {
        let expected = pair(id, (4, 1, 9), (9, 2, 10));
        let offset = crate::ide::position_to_offset(SOURCE, line, character).unwrap();
        let oracle =
            crate::ide::linked_editing::LinkedEditingService::ranges(SOURCE, "/App.vue", offset)
                .unwrap();
        assert_eq!(serde_json::to_value(oracle).unwrap(), expected["result"]);
        assert_eq!(
            send(&mut native, linked(id, line, character)),
            Some(expected)
        );
    }
    assert_eq!(
        send(&mut native, linked(7, 6, 6)),
        Some(pair(7, (6, 5, 8), (6, 32, 35)))
    );
}

#[test]
fn original_whole_unicode_crlf_nonzero_frame_and_invalid_utf16_keep_rpc_coordinates() {
    let mut native = service(true, true);
    open(
        &mut native,
        "<!--😀日本語-->\r\n<style>.x{}</style>\r\n<template>\r\n<Widget>😀</Widget>\r\n</template \t>",
        1,
    );
    assert_eq!(
        send(&mut native, linked(2, 2, 1)),
        Some(pair(2, (2, 1, 9), (4, 2, 10)))
    );
    assert_eq!(
        send(&mut native, linked(3, 4, 10)),
        Some(pair(3, (2, 1, 9), (4, 2, 10)))
    );
    assert_eq!(
        send(&mut native, linked(4, 3, 2)),
        Some(pair(4, (3, 1, 7), (3, 12, 18)))
    );
    assert_eq!(
        send(&mut native, linked(5, 3, 9)),
        Some(error(5, -32602, "Invalid native UTF-16 position"))
    );
    for (id, line, character) in [(6, 1, 2), (7, 2, 0), (8, 4, 1), (9, 4, 11)] {
        assert_eq!(
            send(&mut native, linked(id, line, character)),
            Some(json!({"jsonrpc":"2.0","id":id,"result":null}))
        );
    }
}

#[test]
fn original_frame_case_mismatch_has_whole_null_but_preserves_genuine_inner_ranges() {
    let mut native = service(true, true);
    open(&mut native, "<template><p></p></TeMPLATE >", 1);
    for (id, character) in [(2, 1), (3, 9), (4, 19), (5, 27)] {
        assert_eq!(
            send(&mut native, linked(id, 0, character)),
            Some(json!({"jsonrpc":"2.0","id":id,"result":null}))
        );
    }
    assert_eq!(
        send(&mut native, linked(6, 0, 11)),
        Some(pair(6, (0, 11, 12), (0, 15, 16)))
    );
}

#[test]
fn original_recovered_or_unadmitted_body_refuses_outer_frame_with_complete_error() {
    let mut native = service(true, true);
    for (version, source) in [
        (1, "<template><p></p><div title='unterminated</template>"),
        (2, "<template/>"),
        (3, "<template>x"),
        (4, "<template src='other.html'></template>"),
    ] {
        open(&mut native, source, version);
        for id in [2, 3] {
            assert_eq!(
                send(&mut native, linked(id, 0, 1)),
                Some(error(id, -32012, "Native original template names refused"))
            );
        }
    }
}

#[test]
fn original_frame_change_close_reopen_and_empty_body_keep_complete_current_envelopes() {
    let mut native = service(true, true);
    open(&mut native, SOURCE, 1);
    assert_eq!(
        send(&mut native, linked(2, 4, 3)),
        Some(pair(2, (4, 1, 9), (9, 2, 10)))
    );
    assert_eq!(
        send(
            &mut native,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"<template></template>"}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut native, linked(3, 0, 1)),
        Some(pair(3, (0, 1, 9), (0, 12, 20)))
    );
    assert_eq!(
        send(
            &mut native,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut native, linked(4, 0, 1)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(&mut native, SOURCE, 1);
    assert_eq!(
        send(&mut native, linked(5, 9, 2)),
        Some(pair(5, (4, 1, 9), (9, 2, 10)))
    );
}

#[test]
fn original_frame_cancel_and_opt_in_disable_prevent_pending_pair_publication() {
    let mut native = service(true, true);
    open(&mut native, SOURCE, 1);
    let mut pending = Box::pin(native.call(request(linked(9, 4, 3))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    assert_eq!(
        send(
            &mut native,
            json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":9}})
        ),
        None
    );
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Canceled")
    );
    assert_eq!(
        send(&mut native, linked(10, 4, 3)),
        Some(pair(10, (4, 1, 9), (9, 2, 10)))
    );
    let mut pending = Box::pin(native.call(request(linked(11, 9, 2))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    native
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":false})));
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        json!({"jsonrpc":"2.0","id":11,"error":{"code":-32801,"message":"Content modified"}})
    );
    assert_eq!(
        send(&mut native, linked(12, 4, 3)),
        Some(pair(12, (4, 1, 9), (9, 2, 10)))
    );
}
