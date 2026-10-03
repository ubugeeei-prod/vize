use super::{
    Service, URI, Value, block_on, definition, error, location, open, request, send, service,
};
use serde_json::json;
use std::{future::Future, task::Context};

fn references(id: i32, line: u32, character: u32, include: bool) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeReferences","params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character},"context":{"includeDeclaration":include}}})
}

#[test]
fn original_jsx_and_tsx_component_container_queries_have_complete_envelopes() {
    for language in ["javascriptreact", "typescriptreact"] {
        let mut service = service();
        open(
            &mut service,
            "import UI from 'dep';\nconst value=1;\nconst tree=<UI.Button prop={value}>{value}</UI.Button>;\nvalue;",
            language,
            1,
        );
        assert_eq!(
            send(&mut service, definition(2, 2, 13)),
            Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,7,9)}))
        );
        assert_eq!(
            send(&mut service, references(3, 1, 7, true)),
            Some(
                json!({"jsonrpc":"2.0","id":3,"result":[location(1,6,11),location(2,28,33),location(2,36,41),location(3,0,5)]})
            )
        );
        assert_eq!(
            send(&mut service, references(4, 0, 8, false)),
            Some(json!({"jsonrpc":"2.0","id":4,"result":[location(2,12,14)]}))
        );
        for (id, at) in [(5, 16), (6, 44), (7, 47)] {
            assert_eq!(
                send(&mut service, definition(id, 2, at)),
                Some(json!({"jsonrpc":"2.0","id":id,"result":null}))
            );
        }
    }
}

#[test]
fn tsx_unicode_type_coordinates_and_original_refusals_have_whole_responses() {
    let mut service = service();
    open(
        &mut service,
        "/*😀*/ const café:number=1;\r\nconst tree=<div>{café}</div>;",
        "typescriptreact",
        1,
    );
    assert_eq!(
        send(&mut service, definition(2, 1, 18)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,13,17)}))
    );
    assert_eq!(
        send(&mut service, references(3, 1, 18, true)),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[location(0,13,17),location(1,17,21)]}))
    );
    assert_eq!(
        send(&mut service, definition(4, 0, 3)),
        Some(error(4, -32602, "Invalid native UTF-16 position"))
    );
    open(&mut service, "const tree=<div/>;", "javascript", 1);
    assert_eq!(
        send(&mut service, definition(5, 0, 7)),
        Some(error(5, -32002, "Native original Program refused"))
    );
    open(&mut service, "const tree=<Missing/>;", "typescriptreact", 1);
    assert_eq!(
        send(&mut service, definition(6, 0, 7)),
        Some(error(6, -32003, "Native File observation refused"))
    );
    open(&mut service, "const tree=<div>", "javascriptreact", 1);
    assert_eq!(
        send(&mut service, definition(7, 0, 7)),
        Some(error(7, -32002, "Native original Program refused"))
    );
}

#[test]
fn actual_jsx_change_close_reopen_and_profile_change_preserve_whole_envelopes() {
    let mut service = service();
    open(
        &mut service,
        "const value=1;\nconst tree=<div>{value}</div>;",
        "javascriptreact",
        1,
    );
    assert_eq!(
        send(&mut service, definition(2, 1, 18)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,6,11)}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":"const fresh=1;\nconst tree=<div>{fresh}</div>;"}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, references(3, 1, 18, true)),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[location(0,6,11),location(1,17,22)]}))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, definition(4, 1, 18)),
        Some(error(4, -32602, "Native document unavailable"))
    );
    open(
        &mut service,
        "const value:number=1;\nconst tree=<div>{value}</div>;",
        "typescriptreact",
        1,
    );
    assert_eq!(
        send(&mut service, definition(5, 1, 18)),
        Some(json!({"jsonrpc":"2.0","id":5,"result":location(0,6,11)}))
    );
}

#[test]
fn wire_cancel_and_actual_react_language_replacement_refuse_pending_publication() {
    for host_change in [false, true] {
        let mut service = service();
        open(
            &mut service,
            "const value=1;\nconst tree=<div>{value}</div>;",
            "javascriptreact",
            1,
        );
        let mut pending = Box::pin(service.call(request(definition(9, 1, 18))));
        assert!(
            pending
                .as_mut()
                .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                .is_pending()
        );
        if host_change {
            open(
                &mut service,
                "const value:number=1;\nconst tree=<div>{value}</div>;",
                "typescriptreact",
                1,
            );
        } else {
            assert_eq!(
                send(
                    &mut service,
                    json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":9}})
                ),
                None
            );
        }
        let response = serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap();
        assert_eq!(
            response,
            error(
                9,
                -32800,
                if host_change {
                    "Native query cancelled"
                } else {
                    "Canceled"
                }
            )
        );
        assert_eq!(
            send(&mut service, definition(10, 1, 18)),
            Some(json!({"jsonrpc":"2.0","id":10,"result":location(0,6,11)}))
        );
    }
}
