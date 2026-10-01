//! Configured allowances and severity reach an actual stdio editor host.
use super::{
    RULE, STRICT,
    lsp_process::{LspProcess, file_uri},
    project,
};
use serde_json::{Value, json};
use std::fs;

#[test]
fn native_boolean_rule_options_and_severity_reach_the_lsp() {
    let source = include_str!("../fixtures/strict-boolean/DomElement.vue");
    let Some(project) = project(
        source,
        serde_json::from_str(STRICT).expect("strict options"),
    ) else {
        return;
    };
    let config_path = project.path().join("vize.config.json");
    let mut config: Value =
        serde_json::from_slice(&fs::read(&config_path).expect("config")).expect("JSON");
    config["linter"]["rules"][RULE] = json!("warn");
    config["lsp"] = json!({ "lint": true, "typecheck": false, "ecosystem": false });
    fs::write(
        config_path,
        serde_json::to_vec(&config).expect("LSP config"),
    )
    .expect("config");
    let uri = file_uri(&project.path().join("App.vue"));
    let mut lsp = LspProcess::spawn(project.path());
    lsp.send(
        json!({ "jsonrpc":"2.0", "id":1, "method":"initialize", "params": {
            "processId": null, "rootUri": file_uri(project.path()), "capabilities":{},
            "initializationOptions": { "lint":true, "typecheck":false, "ecosystem":false }
        }}),
    );
    assert!(lsp.recv_response(1)["result"].is_object());
    lsp.send(json!({ "jsonrpc":"2.0", "method":"initialized", "params":{} }));
    lsp.send(
        json!({ "jsonrpc":"2.0", "method":"textDocument/didOpen", "params":{
            "textDocument": { "uri":uri, "languageId":"vue", "version":1, "text":source }
        }}),
    );
    let diagnostic = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
            && message["params"]["version"] == 1
    });
    assert_eq!(
        diagnostic["params"]["diagnostics"],
        json!([{
            "range": { "start": { "line":6, "character":7 }, "end": { "line":6, "character":9 } },
            "severity":2, "code":RULE,
            "codeDescription": { "href":"https://github.com/ubugeeei-prod/vize/blob/main/docs/content/rules/type-and-script.md" },
            "source":"vize/lint",
            "message":"Unexpected nullable object value in conditional. Please handle the nullish case explicitly."
        }])
    );
    lsp.send(json!({ "jsonrpc":"2.0", "id":2, "method":"shutdown", "params":null }));
    assert_eq!(lsp.recv_response(2)["result"], Value::Null);
    lsp.send(json!({ "jsonrpc":"2.0", "method":"exit", "params":null }));
    assert_eq!(lsp.wait_for_exit().code(), Some(0));
}
