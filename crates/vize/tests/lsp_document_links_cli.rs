#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole JSON and filesystem test fixtures"
)]
#![expect(clippy::disallowed_macros, reason = "fixture SHA-256 hex strings")]

use std::{fs, path::Path};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::{LspProcess, file_uri};

const CORPUS: &str = "tests/_fixtures/differential/lsp/document-link-module-resolution-original";
const CASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/document-link-module-resolution-original/case.json"
));
const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/document-link-module-resolution-original/controls.json"
));

#[test]
fn document_links_replay_original_directory_alias_and_encoded_uri() {
    let case: Value = serde_json::from_str(CASE).unwrap();
    let expected = pinned_json(&case["expected"]);
    for crlf in [false, true] {
        let project = tempfile::Builder::new()
            .prefix("vize-link-original-[id]-名前-")
            .tempdir()
            .unwrap();
        for file in case["files"].as_array().unwrap() {
            let bytes = pinned_bytes(file);
            let source = std::str::from_utf8(&bytes).unwrap();
            write(
                project.path(),
                file["runtimePath"].as_str().unwrap(),
                source,
            );
        }
        run_session(
            &project.path().canonicalize().unwrap(),
            "src/App.vue",
            expected.clone(),
            crlf,
            &case["files"],
        );
    }
}

#[test]
fn document_links_keep_whole_module_and_unresolved_controls() {
    let controls: Value = serde_json::from_str(CONTROLS).unwrap();
    for crlf in [false, true] {
        let project = tempfile::Builder::new()
            .prefix("vize-link-controls-[id]-名前-")
            .tempdir()
            .unwrap();
        for file in controls["files"].as_array().unwrap() {
            let text = file["text"].as_str().unwrap();
            assert_eq!(digest(text.as_bytes()), file["sha256"].as_str().unwrap());
            write(project.path(), file["runtimePath"].as_str().unwrap(), text);
        }
        for directory in controls["directories"].as_array().unwrap() {
            fs::create_dir_all(project.path().join(directory.as_str().unwrap())).unwrap();
        }
        for session in controls["sessions"].as_array().unwrap() {
            run_session(
                &project.path().canonicalize().unwrap(),
                session["entry"].as_str().unwrap(),
                session["expected"].clone(),
                crlf,
                &controls["files"],
            );
        }
    }
}

fn run_session(project: &Path, entry: &str, mut expected: Value, crlf: bool, files: &Value) {
    let source_path = project.join(entry);
    let source = fs::read_to_string(&source_path).unwrap();
    let source = if crlf {
        source.replace('\n', "\r\n")
    } else {
        source
    };
    let uri = file_uri(&source_path);
    let workspace_uri = file_uri(project);
    materialize(&mut expected, &workspace_uri);
    let mut lsp = LspProcess::spawn(project);
    lsp.send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":workspace_uri,"capabilities":{},
            "initializationOptions":{"editor":true,"lint":false,"typecheck":false}
        }}),
    );
    let initialize = lsp.recv_response(1);
    assert!(initialize["result"].is_object(), "{initialize:#}");
    assert!(initialize.get("error").is_none(), "{initialize:#}");
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}
        }}),
    );
    let publication = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri
            && message["params"]["version"] == 1
    });
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
        "params":{"uri":uri,"version":1,"diagnostics":[]}})
    );

    // The repeated request must retain the complete ordered response, not just
    // the matching link, and must not reinterpret already-encoded percent bytes.
    for id in [2, 3] {
        lsp.send(
            json!({"jsonrpc":"2.0","id":id,"method":"textDocument/documentLink",
            "params":{"textDocument":{"uri":uri}}}),
        );
        let response = lsp.recv_response(id);
        assert_eq!(
            response,
            json!({"jsonrpc":"2.0","id":id,"result":expected}),
            "entry={entry}, CRLF={crlf}"
        );
        if let Some(links) = response["result"].as_array() {
            for link in links {
                let target = lsp_types::Url::parse(link["target"].as_str().unwrap()).unwrap();
                let path = target.to_file_path().unwrap();
                if path.file_name().unwrap() == "not-yet-created" {
                    assert!(!path.exists(), "original missing-relative contract");
                } else {
                    assert!(path.is_file(), "the link must jump to a file: {target}");
                    // Verify physical UTF-8 target identity as well as the exact
                    // URL response; decoding [id], Unicode and literal %20 must
                    // retrieve the original authored bytes.
                    assert_eq!(file_uri(&path), target.as_str());
                    let relative = path.strip_prefix(project).unwrap();
                    let original = files
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|file| Path::new(file["runtimePath"].as_str().unwrap()) == relative)
                        .expect("the target belongs to the authored input packet");
                    let bytes = original.get("text").map_or_else(
                        || pinned_bytes(original),
                        |text| text.as_str().unwrap().as_bytes().to_vec(),
                    );
                    assert_eq!(fs::read(&path).unwrap(), bytes);
                }
            }
        }
    }
    lsp.send(json!({"jsonrpc":"2.0","id":4,"method":"shutdown"}));
    assert_eq!(
        lsp.recv_response(4),
        json!({"jsonrpc":"2.0","id":4,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(
        lsp.wait_for_exit().success(),
        "exit keeps client stdin open"
    );
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

fn pinned_json(reference: &Value) -> Value {
    serde_json::from_slice(&pinned_bytes(reference)).unwrap()
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
    format!("{:x}", Sha256::digest(bytes))
}

fn write(project: &Path, runtime_path: &str, source: &str) {
    let path = project.join(runtime_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
