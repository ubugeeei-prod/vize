use super::super::request as wire_request;
use super::{error, json, occurrence, open, request, send, service};
use crate::runtime::block_on;
use std::{future::Future, task::Context};
use tower::Service;

#[test]
fn real_wire_cancel_and_host_change_refuse_highlight_publication_in_both_request_families() {
    for (method, source, language) in [
        (
            "vize/nativeDocumentHighlight",
            "let value=1;value++;",
            "javascript",
        ),
        (
            "vize/nativeTemplateDocumentHighlight",
            "<template><button @click='let value=$event;return value'/></template>",
            "vue",
        ),
    ] {
        for host_change in [false, true] {
            let mut service = service();
            open(&mut service, source, language);
            let at = occurrence(source, "value", 0, 1).0;
            let mut pending = Box::pin(service.call(wire_request(request(9, method, at))));
            assert!(
                pending
                    .as_mut()
                    .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                    .is_pending()
            );
            if host_change {
                open(&mut service, source, language);
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
                send(&mut service, request(10, method, at)),
                Some(
                    json!({"jsonrpc":"2.0","id":10,"result":[occurrence(source,"value",0,1).1,occurrence(source,"value",1,if language=="javascript" {3} else {2}).1]})
                )
            );
        }
    }
}
