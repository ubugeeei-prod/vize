#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole JSON and original physical test inputs"
)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::{LspProcess, file_uri};

const CORPUS: &str = "tests/_fixtures/differential/lsp/binding-occurrences-original";
const CASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/binding-occurrences-original/case.json"
));
const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/binding-occurrences-original/controls.authored.json"
));

#[test]
fn original_whole_highlights_references_and_lenses_replay_over_default_stdio() {
    let case: Value = serde_json::from_str(CASE).unwrap();
    let requests: Value = serde_json::from_slice(&pinned_bytes(&case["expected"])).unwrap();
    let input = pinned_bytes(&case["files"][0]);
    let config = pinned_bytes(&case["files"][1]);
    for crlf in [false, true] {
        let project = tempfile::Builder::new()
            .prefix("vize-7992-original-名前-")
            .tempdir()
            .unwrap();
        fs::write(project.path().join("Field.vue"), &input).unwrap();
        fs::write(project.path().join("tsconfig.json"), &config).unwrap();
        run_session(
            project.path(),
            "Field.vue",
            std::str::from_utf8(&input).unwrap(),
            &requests,
            crlf,
        );
    }
}

#[test]
fn whole_shadow_css_unicode_art_and_refusal_responses_use_physical_coordinates() {
    let controls: Value = serde_json::from_str(CONTROLS).unwrap();
    for session in controls["sessions"].as_array().unwrap() {
        let source = session["source"].as_str().unwrap();
        assert_eq!(
            digest(source.as_bytes()),
            session["sourceSha256"].as_str().unwrap()
        );
        for crlf in [false, true] {
            let project = tempfile::Builder::new()
                .prefix("vize-7992-ownership-名前-")
                .tempdir()
                .unwrap();
            let entry = session["entry"].as_str().unwrap();
            fs::write(project.path().join(entry), source).unwrap();
            run_session(project.path(), entry, source, &session["requests"], crlf);
        }
    }
}

fn run_session(project: &Path, entry: &str, input: &str, requests: &Value, crlf: bool) {
    let project = project.canonicalize().unwrap();
    let uri = file_uri(&project.join(entry));
    let workspace = file_uri(&project);
    let source = if crlf {
        input.replace('\n', "\r\n")
    } else {
        input.into()
    };
    let mut lsp = LspProcess::spawn(&project);
    lsp.send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":workspace,"capabilities":{},
            "initializationOptions":{"editor":true,"lint":false,"typecheck":false}
        }}),
    );
    let initialized = lsp.recv_response(1);
    assert!(initialized["result"].is_object(), "{initialized:#}");
    assert!(initialized.get("error").is_none(), "{initialized:#}");
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
        "textDocument":{"uri":uri,"languageId":if entry.ends_with(".art.vue") {"art-vue"} else {"vue"},"version":1,"text":source}
    }}));
    let publication = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"].as_str() == Some(uri.as_str())
            && message["params"]["version"] == 1
    });
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
        "params":{"uri":uri,"version":1,"diagnostics":[]}})
    );
    let mut id = 2;
    // Repeated requests compare every range/kind, URI, command and omission.
    // No response is filtered, projected, or used as the next oracle.
    for _ in 0..2 {
        for request in requests.as_array().unwrap() {
            let mut params = request["params"].clone();
            params
                .as_object_mut()
                .unwrap()
                .insert("textDocument".into(), json!({"uri":uri}));
            let mut expected = request["result"].clone();
            materialize(&mut expected, workspace.as_str());
            lsp.send(json!({"jsonrpc":"2.0","id":id,"method":request["method"],"params":params}));
            assert_eq!(
                lsp.recv_response(id),
                json!({"jsonrpc":"2.0","id":id,"result":expected}),
                "entry={entry}, CRLF={crlf}"
            );
            id += 1;
        }
    }
    lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
    assert_eq!(
        lsp.recv_response(id),
        json!({"jsonrpc":"2.0","id":id,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

fn pinned_bytes(reference: &Value) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let bytes = fs::read(
        root.join(CORPUS)
            .join(reference["source"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(digest(&bytes), reference["sha256"].as_str().unwrap());
    bytes
}

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for byte in Sha256::digest(bytes) {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn materialize(value: &mut Value, workspace: &str) {
    match value {
        Value::String(text) => *text = text.replace("${workspace}", workspace),
        Value::Array(values) => values
            .iter_mut()
            .for_each(|value| materialize(value, workspace)),
        Value::Object(values) => values
            .values_mut()
            .for_each(|value| materialize(value, workspace)),
        _ => {}
    }
}
