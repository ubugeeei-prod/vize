//! The pinned template-ref key fixtures, replayed through the production
//! service builder the stdio server uses.

use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request};

use crate::runtime::block_on;
use crate::server::{MaestroServer, build_lsp_service};

const WORKSPACE: &str = "file:///workspace";

macro_rules! fixture {
    ($name:literal, $source:literal, $expected:literal) => {
        (
            include_str!(concat!(
                "../../../../../../tests/_fixtures/differential/lsp/",
                $name,
                "/",
                $source
            )),
            include_str!(concat!(
                "../../../../../../tests/_fixtures/differential/lsp/",
                $name,
                "/",
                $expected
            )),
        )
    };
}

fn send(service: &mut LspService<MaestroServer>, value: Value) -> Option<Value> {
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service
            .call(serde_json::from_value::<Request>(value).unwrap())
            .await
            .unwrap()
            .map(|response| serde_json::to_value(response).unwrap())
    })
}

/// Every request of a fixture as `(method, params, result)`.
fn requests(method: &str, expected: Value) -> Vec<(String, Value, Value)> {
    match expected {
        Value::Array(requests) => requests
            .into_iter()
            .map(|request| {
                let method = request
                    .get("method")
                    .and_then(Value::as_str)
                    .unwrap_or(method)
                    .to_owned();
                (method, request["params"].clone(), request["result"].clone())
            })
            .collect(),
        single => vec![(
            method.to_owned(),
            json!({ "position": single["position"] }),
            single["result"].clone(),
        )],
    }
}

fn replay(entry: &str, source: &str, method: &str, expected: &str) {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    send(
        &mut service,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"editor":true,"lint":false,"typecheck":false}}}),
    )
    .unwrap();
    send(
        &mut service,
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    );
    let uri = [WORKSPACE, "/", entry].concat();
    send(
        &mut service,
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}}}),
    );
    let expected: Value =
        serde_json::from_str(&expected.replace("${workspace}", WORKSPACE)).unwrap();
    for (id, (method, mut params, result)) in (2..).zip(requests(method, expected)) {
        params["textDocument"] = json!({ "uri": uri });
        let response = send(
            &mut service,
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
        )
        .unwrap();
        assert_eq!(response["result"], result, "{entry} {method}");
    }
}

#[test]
fn pinned_fixtures_answer_over_the_wire() {
    for ((source, expected), method, entry) in [
        (
            fixture!(
                "template-ref-key-refs-rename",
                "App.vue.txt",
                "responses.expected.json"
            ),
            "textDocument/rename",
            "App.vue",
        ),
        (
            fixture!(
                "template-ref-key-bound-literal-references",
                "App.vue.txt",
                "responses.expected.json"
            ),
            "textDocument/references",
            "App.vue",
        ),
        (
            fixture!(
                "template-ref-key-element-definitions",
                "App.vue.txt",
                "response.expected.json"
            ),
            "textDocument/definition",
            "App.vue",
        ),
        (
            fixture!(
                "template-ref-key-definition",
                "KeyedRef.vue.txt",
                "response.expected.json"
            ),
            "textDocument/definition",
            "KeyedRef.vue",
        ),
    ] {
        replay(entry, source, method, expected);
    }
}
