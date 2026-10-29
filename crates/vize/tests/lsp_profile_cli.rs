#![cfg(feature = "maestro")]
//! The profile stays outside stdio protocol frames and survives normal exit.

use serde_json::{Value, json};

use crate::support::lsp_process::{LspProcess, file_uri};

#[test]
fn lsp_exports_profile_after_graceful_stdio_shutdown() {
    let project = tempfile::tempdir().unwrap();
    let source = "<script setup>const value = 1;</script><template>{{ value }}</template>";
    let document = project.path().join("App.vue");
    std::fs::write(&document, source).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), "{}").unwrap();
    let profile = project.path().join("lsp-profile.json");
    let uri = file_uri(&document);
    let mut lsp = LspProcess::spawn_profiled(project.path(), &profile);
    lsp.send(json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "processId": null, "rootUri": file_uri(project.path()), "capabilities": {},
            "initializationOptions": {
                "editor": true, "lint": false, "typecheck": false, "ecosystem": false
            }
        }
    }));
    assert!(lsp.recv_response(1)["result"].is_object());
    lsp.send(json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    lsp.send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": {"textDocument": {
            "uri": uri, "languageId": "vue", "version": 1, "text": source
        }}
    }));
    lsp.send(json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/documentSymbol",
        "params": {"textDocument": {"uri": uri}}
    }));
    let symbols = lsp.recv_response(2);
    let symbols = symbols["result"].as_array().unwrap();
    assert!(
        symbols
            .iter()
            .any(|symbol| symbol["name"] == "script setup")
    );
    assert!(symbols.iter().any(|symbol| symbol["name"] == "template"));
    assert!(
        !profile.exists(),
        "a live session has not exported a terminal profile"
    );
    lsp.send(json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}));
    assert!(lsp.recv_response(3)["result"].is_null());
    lsp.send(json!({"jsonrpc": "2.0", "method": "exit"}));
    assert!(lsp.wait_for_exit().success());
    let report: Value = serde_json::from_slice(&std::fs::read(profile).unwrap()).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["command"], "lsp");
    assert!(report["spans"].is_array());
    assert!(report["counters"].is_array());
    #[cfg(not(feature = "profiling"))]
    assert!(
        report["allocation"].is_null(),
        "the ordinary allocator has no counts"
    );
    #[cfg(feature = "profiling")]
    {
        assert!(report["allocation"]["calls"].as_u64().unwrap() > 0);
        let operations = report["micro"].as_array().unwrap();
        assert!(operations.iter().any(|row| {
            row["name"] == "maestro.project.document_root" && row["calls"].as_u64().unwrap() > 0
        }));
        assert!(operations.iter().any(|row| {
            row["name"] == "fs.read_dir.entries" && row["units"].as_u64().unwrap() > 0
        }));
    }
}
