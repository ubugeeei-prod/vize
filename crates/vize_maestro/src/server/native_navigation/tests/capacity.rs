use super::{error, json, send, service};

#[test]
fn native_worker_capacity_has_a_complete_json_rpc_refusal_without_eviction() {
    let mut service = service();
    for index in 0..17 {
        let uri = vize_l0::cstr!("file:///wire-capacity-{index}.js");
        assert_eq!(
            send(
                &mut service,
                json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"javascript","version":1,"text":"const value=1;value;"}}})
            ),
            None
        );
        let id = index + 2;
        let response = send(
            &mut service,
            json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeDefinition","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":15}}}),
        );
        let expected = if index < 16 {
            json!({"jsonrpc":"2.0","id":id,"result":{"uri":uri,"range":{"start":{"line":0,"character":6},"end":{"line":0,"character":11}}}})
        } else {
            error(id, -32007, "Native navigation worker capacity reached")
        };
        assert_eq!(response, Some(expected));
    }
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":20,"method":"vize/nativeDefinition","params":{"textDocument":{"uri":"file:///wire-capacity-0.js"},"position":{"line":0,"character":15}}})
        ),
        Some(
            json!({"jsonrpc":"2.0","id":20,"result":{"uri":"file:///wire-capacity-0.js","range":{"start":{"line":0,"character":6},"end":{"line":0,"character":11}}}})
        )
    );
}
