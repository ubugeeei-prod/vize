//! Real service requests never replace unavailable checker types with syntax.

use crate::{
    runtime::block_on,
    server::{MaestroServer, build_lsp_service},
};
use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request};

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/computed-inlay-hints/App.vue.txt"
);

fn send(service: &mut LspService<MaestroServer>, value: Value) -> Option<Value> {
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service
            .call(serde_json::from_value::<Request>(value).unwrap())
            .await
            .unwrap()
            .map(|reply| serde_json::to_value(reply).unwrap())
    })
}

#[test]
fn disabled_checker_has_no_guessed_type_hint_and_keeps_the_original_buffer() {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let init = send(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"typecheck":false,"lint":false,"inlayHints":true,"ecosystem":false}}})).unwrap();
    assert!(init["result"].is_object());
    let uri = "file:///App.vue";
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
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":ORIGINAL}}})
        ),
        None
    );
    let query = json!({"jsonrpc":"2.0","id":2,"method":"textDocument/inlayHint","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":100,"character":0}}}});
    assert_eq!(
        send(&mut service, query),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
    let uri = tower_lsp::lsp_types::Url::parse(uri).unwrap();
    assert_eq!(
        service.inner().state.documents.text(&uri).as_deref(),
        Some(ORIGINAL)
    );
    #[cfg(feature = "native")]
    assert!(!service.inner().state.has_corsa_bridge());
}

#[cfg(feature = "native")]
#[test]
fn unavailable_checker_does_not_render_the_reported_placeholder_types() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("vize.config.json"),
        serde_json::to_vec(
            &json!({"typeChecker":{"corsaPath":root.path().join("missing-native")}}),
        )
        .unwrap(),
    )
    .unwrap();
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let root_uri = tower_lsp::lsp_types::Url::from_directory_path(root.path()).unwrap();
    let uri = root_uri.join("App.vue").unwrap();
    let init = send(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":root_uri,"capabilities":{},"initializationOptions":{"typecheck":true,"lint":false,"inlayHints":true,"ecosystem":false}}})).unwrap();
    assert!(init["result"].is_object());
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
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":ORIGINAL}}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":2,"method":"textDocument/inlayHint","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":11,"character":0}}}})
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
    assert!(service.inner().state.corsa_init_failure().is_some());
    assert_eq!(
        service.inner().state.documents.text(&uri).as_deref(),
        Some(ORIGINAL)
    );
}
