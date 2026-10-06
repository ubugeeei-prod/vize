#![cfg(all(test, unix))]

use serde_json::{Value, json};
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[path = "support/lsp_symlink_workspace_fixture.rs"]
mod fixture;
#[path = "support/lsp_process.rs"]
mod lsp_process;
use fixture::{Fixture, Layout};
use lsp_process::{LspProcess, file_uri};

#[test]
fn original_two_queued_opens_never_write_into_the_authored_workspace_package() {
    // Preserve the reporter's three runs and original 15-second live interval.
    for _ in 0..3 {
        run(Layout::Original, true, false, true);
    }
}

#[test]
fn original_include_and_open_order_controls_keep_all_authored_bytes() {
    run(Layout::Original, false, false, false);
    run(Layout::BroadInclude, true, false, false);
    run(Layout::SrcInclude, true, false, false);
    run(Layout::Original, true, true, false);
    run(Layout::CrLf, true, false, false);
    run(Layout::AuthoredCompanions, true, false, false);
}

fn run(layout: Layout, open_other: bool, reverse: bool, original_wait: bool) {
    let fixture = Fixture::new(layout);
    let mut process = LspProcess::spawn(&fixture.app);
    process.send(
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "processId": std::process::id(), "rootUri": file_uri(&fixture.app), "capabilities": {}
        }}),
    );
    let initialize = receive(&mut process, |message| message["id"] == 1);
    assert!(initialize.get("error").is_none(), "{initialize:#}");
    assert!(initialize["result"].is_object(), "{initialize:#}");
    process.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    // Do not wait for diagnostics between these opens: the original shared
    // complete overlay revision is what exposes the former package link.
    if open_other && !reverse {
        open(&mut process, &fixture.other, &fixture.other_source, "vue");
    }
    open(&mut process, &fixture.host, &fixture.host_source, "vue");
    if open_other && reverse {
        open(&mut process, &fixture.other, &fixture.other_source, "vue");
    }
    if original_wait {
        std::thread::sleep(Duration::from_secs(15));
    }
    publication(&mut process, &fixture.host, 1, json!([]));
    if open_other {
        publication(&mut process, &fixture.other, 1, json!([]));
    }
    fixture.assert_authored_unchanged();

    // A separate unsaved plain-TS control proves that an unavailable native
    // backend cannot make the empty original Vue publications pass vacuously.
    let probe = fixture.app.join("NativeProbe.ts");
    open(
        &mut process,
        &probe,
        "export const value: number = 'wrong';\n",
        "typescript",
    );
    publication(
        &mut process,
        &probe,
        1,
        json!([{
            "range": { "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 18 } },
            "severity": 1, "code": 2322, "source": "vize/types",
            "message": "Type 'string' is not assignable to type 'number'."
        }]),
    );
    process.send(
        json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": { "uri": file_uri(&probe), "version": 2 },
            "contentChanges": [{ "text": "export const value: number = 1;\n" }]
        }}),
    );
    publication(&mut process, &probe, 2, json!([]));
    fixture.assert_authored_unchanged();
    if original_wait {
        // The exact reporter awaits any reply, without asserting success.
        // Pinned tower-lsp rejects present params for a no-params request.
        process.send(json!({ "jsonrpc": "2.0", "id": 9, "method": "shutdown", "params": null }));
        assert_eq!(
            receive(&mut process, |message| message["id"] == 9),
            json!({ "jsonrpc": "2.0", "id": 9, "error": {
                "code": -32602, "message": "Unexpected params: null"
            } })
        );
        process.send(json!({ "jsonrpc": "2.0", "method": "exit", "params": null }));
    } else {
        // Independently retain successful shutdown for valid authored clients.
        process.send(json!({ "jsonrpc": "2.0", "id": 9, "method": "shutdown" }));
        assert_eq!(
            receive(&mut process, |message| message["id"] == 9),
            json!({ "jsonrpc": "2.0", "id": 9, "result": null })
        );
        process.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
    }
    assert!(process.wait_for_exit().success());
    fixture.assert_authored_unchanged();
}

fn open(process: &mut LspProcess, path: &Path, source: &str, language_id: &str) {
    process.send(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
        "textDocument": { "uri": file_uri(path), "languageId": language_id, "version": 1, "text": source }
    }}));
}

fn publication(process: &mut LspProcess, path: &Path, version: i32, diagnostics: Value) {
    let uri = file_uri(path);
    let matches = |message: &Value| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
            && message["params"]["version"] == version
    };
    let observed = process
        .published_diagnostics()
        .into_iter()
        .find(&matches)
        .unwrap_or_else(|| receive(process, matches));
    assert_eq!(
        observed,
        json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": uri, "version": version, "diagnostics": diagnostics
        }})
    );
}

fn receive(process: &mut LspProcess, matches: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            Instant::now() < deadline,
            "original stdio client reply deadline exceeded"
        );
        let message = process.recv_matching(|_| true);
        // The complete original client answers server -> client requests with
        // null; preserve that behavior instead of changing initialization.
        if message.get("method").is_some() && message.get("id").is_some() {
            process.send(json!({ "jsonrpc": "2.0", "id": message["id"], "result": null }));
        } else if matches(&message) {
            return message;
        }
    }
}
