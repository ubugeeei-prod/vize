use futures::{FutureExt, StreamExt, future};
use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request, lsp_types::Diagnostic};

use super::with_native_diagnostic_timeout;
use crate::{MaestroServer, ide::DiagnosticService};

const SOURCE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/lsp-regressions/native-diagnostic-timeout-7698/App.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/lsp-regressions/native-diagnostic-timeout-7698/expected.authored.json"
);

fn expected() -> Value {
    serde_json::from_str(EXPECTED).expect("whole authored timeout contract")
}

#[test]
fn outer_native_timeout_publishes_the_whole_incomplete_status_instead_of_clean_empty() {
    let root = tempfile::tempdir().expect("public timeout fixture root");
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).expect("whole authored SFC");
    let uri = tower_lsp::lsp_types::Url::from_file_path(path).expect("authored source URI");
    let state =
        super::super::super::editor_typecheck_fixture::state_for_fixture(root.path(), &uri, SOURCE);
    state.update_virtual_docs(&uri, SOURCE);
    let mut diagnostics = DiagnosticService::collect(&state, &uri);
    assert_eq!(json!(diagnostics), expected()["completedClean"]);

    let hints = crate::runtime::block_on(with_native_diagnostic_timeout(future::pending::<
        Vec<Diagnostic>,
    >()))
    .expect_err("the unchanged ten-second production bound must expire");
    diagnostics.extend(hints);

    let mut captured_client = None;
    let (mut service, mut socket) = LspService::new(|client| {
        captured_client = Some(client.clone());
        MaestroServer::new(client)
    });
    let initialize: Request = serde_json::from_value(json!({
        "jsonrpc":"2.0", "id":7698, "method":"initialize",
        "params":{"rootUri":tower_lsp::lsp_types::Url::from_file_path(root.path()).expect("workspace URI"), "capabilities":{}}
    }))
    .expect("whole initialize request");
    crate::runtime::block_on(async {
        future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .expect("LSP service ready");
        let reply = service
            .call(initialize)
            .await
            .expect("initialize transport")
            .expect("initialize reply");
        assert!(serde_json::to_value(reply).expect("whole initialize reply")["error"].is_null());
    });
    let client = captured_client.expect("actual ClientSocket sender");
    let observed = crate::runtime::block_on(async {
        let send = client.publish_diagnostics(uri.clone(), diagnostics, Some(7));
        let receive = socket.next();
        let ((), message) = futures::join!(send, receive);
        serde_json::to_value(message.expect("whole diagnostic notification"))
            .expect("whole notification JSON")
    });
    let mut packet = expected()["timeout"].clone();
    packet["params"]["uri"] = json!(uri);
    assert_eq!(observed, packet);
    assert!(socket.next().now_or_never().is_none());
}

#[test]
fn completed_clean_and_positive_native_vectors_are_unchanged() {
    let expected = expected();
    for key in ["completedClean", "completedNative"] {
        let diagnostics: Vec<Diagnostic> =
            serde_json::from_value(expected[key].clone()).expect("whole authored native vector");
        let result =
            crate::runtime::block_on(with_native_diagnostic_timeout(future::ready(diagnostics)))
                .expect("completed result must retain success");
        assert_eq!(json!(result), expected[key]);
    }
}

#[test]
fn an_answered_native_error_is_not_reclassified_as_an_outer_timeout() {
    let error = vize_canon::CorsaBridgeError::ResponseError {
        code: -32602,
        message: "authored complete backend refusal".into(),
    };
    let result = crate::runtime::block_on(with_native_diagnostic_timeout(future::ready(Err::<
        Vec<Diagnostic>,
        _,
    >(
        error
    ))))
    .expect("answered error must preserve the existing fallback classification");
    match result {
        Err(vize_canon::CorsaBridgeError::ResponseError { code, message }) => {
            assert_eq!(code, -32602);
            assert_eq!(message, "authored complete backend refusal");
        }
        _ => panic!("the answered native error changed classification"),
    }
}
