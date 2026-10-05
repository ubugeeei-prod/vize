#![cfg(unix)]
#![expect(clippy::disallowed_types, reason = "complete JSON-RPC process fixture")]

#[path = "support/lsp_process.rs"]
mod lsp_process;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};

const APP: &str = include_str!("../../../tests/_fixtures/lsp-corsa-responsiveness-8012/App.vue");
const TOAST: &str =
    include_str!("../../../tests/_fixtures/lsp-corsa-responsiveness-8012/useToast.ts");
const PACKAGE: &str =
    include_str!("../../../tests/_fixtures/lsp-corsa-responsiveness-8012/package.json");
const TSCONFIG: &str =
    include_str!("../../../tests/_fixtures/lsp-corsa-responsiveness-8012/tsconfig.json");

fn request(id: Value, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn workspace(root: &Path) {
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/App.vue"), APP).unwrap();
    std::fs::write(root.join("src/useToast.ts"), TOAST).unwrap();
    std::fs::write(root.join("package.json"), PACKAGE).unwrap();
    std::fs::write(root.join("tsconfig.json"), TSCONFIG).unwrap();
    let backend = root.join("held-corsa");
    // The shell retains stdout while cat drains stdin: the actual handshake
    // enters synchronous IPC but cannot finish before the process is released.
    std::fs::write(
        &backend,
        "#!/bin/sh\necho $$ >> backend.pids\n: > backend.entered\ncat > /dev/null\n",
    )
    .unwrap();
    std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(
        root.join("vize.config.json"),
        json!({
            "lsp": { "lint": false, "typecheck": true },
            "typeChecker": { "corsaPath": backend },
        })
        .to_string(),
    )
    .unwrap();
}

fn await_backend(root: &Path) {
    let started = Instant::now();
    while !root.join("backend.entered").exists() {
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the held backend never entered"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

struct BackendCleanup<'a>(&'a Path);
impl BackendCleanup<'_> {
    fn stop(&self) {
        if let Ok(pids) = std::fs::read_to_string(self.0.join("backend.pids")) {
            let _ = std::process::Command::new("kill")
                .args(pids.lines())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}
impl Drop for BackendCleanup<'_> {
    fn drop(&mut self) {
        self.stop();
    }
}

fn start(root: &Path, typecheck: bool) -> (LspProcess, Value) {
    let uri = json!(file_uri(&root.join("src/App.vue")));
    let mut lsp = LspProcess::spawn(root);
    lsp.send(request(json!(1), "initialize", json!({
        "processId": std::process::id(), "rootUri": file_uri(root),
        "capabilities": {}, "initializationOptions": { "editor": true, "hover": true, "typecheck": typecheck, "lint": false },
    })));
    assert!(lsp.recv_response(1).get("result").is_some());
    lsp.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    lsp.send(
        json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": { "uri": uri, "languageId": "vue", "version": 1, "text": APP },
        }}),
    );
    (lsp, uri)
}

fn start_hover(root: &Path, id: Value) -> (LspProcess, Value) {
    let (mut lsp, uri) = start(root, true);
    // The original report's useToast() request starts the native session.
    lsp.send(request(
        id,
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri }, "position": { "line": 3, "character": 20 },
        }),
    ));
    await_backend(root);
    (lsp, uri)
}

fn syntax_control(root: &Path) -> (Value, Value) {
    let (mut lsp, uri) = start(root, false);
    lsp.send(request(
        json!(2),
        "textDocument/documentSymbol",
        json!({"textDocument":{"uri":uri}}),
    ));
    let symbols = lsp.recv_response(2);
    lsp.send(request(
        json!(3),
        "textDocument/foldingRange",
        json!({"textDocument":{"uri":uri}}),
    ));
    let folds = lsp.recv_response(3);
    lsp.send(request(json!(20), "shutdown", Value::Null));
    assert_eq!(
        lsp.recv_response(20),
        json!({"jsonrpc":"2.0","id":20,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
    (symbols, folds)
}

#[test]
fn held_type_startup_preserves_syntax_cancel_and_shutdown_wire_contracts() {
    let root = tempfile::tempdir().unwrap();
    workspace(root.path());
    let _backend = BackendCleanup(root.path());
    let (expected_symbols, expected_folds) = syntax_control(root.path());
    let (mut lsp, uri) = start_hover(root.path(), json!("held-hover"));
    lsp.send(request(
        json!(2),
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": uri } }),
    ));
    let symbols = lsp.recv_response(2);
    assert_eq!(symbols, expected_symbols);
    assert_eq!(symbols["jsonrpc"], "2.0");
    assert_eq!(symbols["id"], 2);
    assert!(symbols.get("error").is_none(), "{symbols}");
    assert!(
        symbols["result"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "{symbols}"
    );
    lsp.send(request(
        json!(3),
        "textDocument/foldingRange",
        json!({ "textDocument": { "uri": uri } }),
    ));
    let folds = lsp.recv_response(3);
    assert_eq!(folds, expected_folds);
    assert_eq!(folds["jsonrpc"], "2.0");
    assert_eq!(folds["id"], 3);
    assert!(folds.get("error").is_none(), "{folds}");
    assert!(
        folds["result"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "{folds}"
    );
    lsp.send(
        json!({ "jsonrpc": "2.0", "method": "$/cancelRequest", "params": { "id": "held-hover" } }),
    );
    assert_eq!(
        lsp.recv_matching(|message| message["id"] == "held-hover"),
        json!({
            "jsonrpc": "2.0", "id": "held-hover", "error": { "code": -32800, "message": "Canceled" },
        })
    );
    // Unknown IDs are ignored; they must not cancel a subsequent cheap query.
    lsp.send(
        json!({ "jsonrpc": "2.0", "method": "$/cancelRequest", "params": { "id": "absent" } }),
    );
    lsp.send(request(
        json!(4),
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": uri } }),
    ));
    assert_eq!(lsp.recv_response(4)["result"], symbols["result"]);
    lsp.send(request(json!(5), "shutdown", Value::Null));
    assert_eq!(
        lsp.recv_response(5),
        json!({ "jsonrpc": "2.0", "id": 5, "result": null })
    );
    lsp.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
    assert!(
        lsp.wait_for_exit().success(),
        "exit must terminate while stdin remains open"
    );
}

#[test]
fn queued_typed_request_cancels_by_numeric_id_while_original_hover_is_held() {
    let root = tempfile::tempdir().unwrap();
    workspace(root.path());
    let _backend = BackendCleanup(root.path());
    let (mut lsp, uri) = start_hover(root.path(), json!("original-hover"));
    lsp.send(request(
        json!(8),
        "textDocument/references",
        json!({
            "textDocument": {"uri":uri}, "position":{"line":3,"character":20},
            "context":{"includeDeclaration":true}
        }),
    ));
    lsp.send(json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":8}}));
    assert_eq!(
        lsp.recv_response(8),
        json!({
            "jsonrpc":"2.0","id":8,"error":{"code":-32800,"message":"Canceled"}
        })
    );
    lsp.send(request(
        json!(9),
        "textDocument/documentSymbol",
        json!({"textDocument":{"uri":uri}}),
    ));
    assert!(
        lsp.recv_response(9)["result"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    // The original hover remains pending; shutdown must not require cancelling
    // it first or waiting for its configured backend bound to expire.
    lsp.send(request(json!(10), "shutdown", Value::Null));
    assert_eq!(
        lsp.recv_response(10),
        json!({"jsonrpc":"2.0","id":10,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

#[test]
fn shutdown_with_uncancelled_original_hover_exits_with_stdin_still_open() {
    let root = tempfile::tempdir().unwrap();
    workspace(root.path());
    let _backend = BackendCleanup(root.path());
    let (mut lsp, _uri) = start_hover(root.path(), json!(17));
    lsp.send(request(json!(18), "shutdown", Value::Null));
    assert_eq!(
        lsp.recv_response(18),
        json!({"jsonrpc":"2.0","id":18,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

#[test]
fn changed_root_and_open_dependency_refuse_the_entire_original_request_on_the_wire() {
    for change_dependency in [false, true] {
        let root = tempfile::tempdir().unwrap();
        workspace(root.path());
        let backend = BackendCleanup(root.path());
        let (mut lsp, uri) = start_hover(root.path(), json!("stale-original"));
        if change_dependency {
            lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":file_uri(&root.path().join("src/useToast.ts")),"languageId":"typescript","version":1,
                    "text":"export function useToast() { return { notify: () => 1 }; }"}
            }}));
        } else {
            lsp.send(
                json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                    "textDocument":{"uri":uri,"version":2},
                    "contentChanges":[{"text":"<template>replacement</template>"}]
                }}),
            );
        }
        // A complete syntax response is the processing barrier: the editor
        // mutation above is applied while the native handshake is still held.
        lsp.send(request(
            json!(30),
            "textDocument/documentSymbol",
            json!({"textDocument":{"uri":uri}}),
        ));
        assert!(lsp.recv_response(30).get("error").is_none());
        // Retire this deliberately non-answering backend. The fixture tests
        // real IPC failure and whole RPC refusal, not a successful native query.
        backend.stop();
        assert_eq!(
            lsp.recv_matching(|message| message["id"] == "stale-original"),
            json!({
                "jsonrpc":"2.0","id":"stale-original","error":{"code":-32801,"message":"Content modified"}
            })
        );
        lsp.send(request(json!(31), "shutdown", Value::Null));
        assert_eq!(
            lsp.recv_response(31),
            json!({"jsonrpc":"2.0","id":31,"result":null})
        );
        lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(lsp.wait_for_exit().success());
    }
}

#[test]
fn direct_exit_terminates_with_uncancelled_native_work_and_open_stdin() {
    let root = tempfile::tempdir().unwrap();
    workspace(root.path());
    let _backend = BackendCleanup(root.path());
    let (mut lsp, _uri) = start_hover(root.path(), json!(41));
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}
