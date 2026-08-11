use super::*;

#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};

#[cfg(unix)]
const MALFORMED_DIAGNOSTIC_SERVER: &str = r#"#!/usr/bin/env python3
import json
import pathlib
import sys

trace = pathlib.Path("fake-server.methods")

def read_message():
    content_length = None
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if line == b"\r\n":
            break
        if line.lower().startswith(b"content-length:"):
            content_length = int(line.split(b":", 1)[1].strip())
    if content_length is None:
        raise RuntimeError("missing Content-Length")
    return json.loads(sys.stdin.buffer.read(content_length))

def write_body(body):
    encoded = body.encode("utf-8")
    sys.stdout.buffer.write(
        f"Content-Length: {len(encoded)}\r\n\r\n".encode("ascii") + encoded
    )
    sys.stdout.buffer.flush()

while True:
    message = read_message()
    if message is None:
        break
    method = message.get("method", "")
    with trace.open("a", encoding="utf-8") as output:
        output.write(method + "\n")
    if method == "exit":
        break
    if "id" not in message:
        continue
    request_id = json.dumps(message["id"], separators=(",", ":"))
    if method == "initialize":
        result = '{"capabilities":{}}'
    elif method == "textDocument/diagnostic":
        # The frame length is exact, but the body has a second JSON value.
        # This models the trailing-body failure class from Content Mapper
        # attempt 1 in #4154.
        write_body(
            f'{{"jsonrpc":"2.0","id":{request_id},"result":{{}}}}{{}}'
        )
        continue
    else:
        result = "null"
    write_body(f'{{"jsonrpc":"2.0","id":{request_id},"result":{result}}}')
"#;

#[test]
#[cfg(unix)]
fn semantic_readiness_does_not_depend_on_pull_diagnostic_framing() {
    let root = tempfile::tempdir().expect("temp root");
    let executable = root.path().join("fake-tsgo");
    fs::write(&executable, MALFORMED_DIAGNOSTIC_SERVER).expect("write fake LSP server");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make fake LSP server executable");
    let document_uri = "file:///workspace/App.vue.ts";
    let documents = FxHashMap::from_iter([(document_uri.into(), "export const value = 1;".into())]);

    let mut session = EditorLspSession::spawn(
        executable.to_str().expect("utf-8 executable"),
        root.path(),
        root.path(),
    )
    .expect("spawn fake editor LSP");
    session
        .synchronize(&documents)
        .expect("open virtual document");
    assert_eq!(
        session.hover(document_uri, 0, 0).expect("semantic hover"),
        None
    );
    session.shutdown().expect("shutdown fake editor LSP");

    let methods =
        fs::read_to_string(root.path().join("fake-server.methods")).expect("read fake LSP methods");
    assert!(!methods.contains("textDocument/diagnostic"), "{methods}");
    assert_eq!(
        methods.matches("textDocument/hover").count(),
        2,
        "{methods}"
    );
}

#[test]
fn signature_help_request_defaults_to_manual_invocation() {
    let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
    let params = signature_help_request_params(&uri, 4, 7, None);

    assert_eq!(
        params["context"],
        serde_json::json!({"triggerKind": 1, "isRetrigger": false})
    );
}

#[test]
fn signature_help_request_preserves_client_context_losslessly() {
    let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
    let context = serde_json::json!({
        "triggerKind": 2,
        "triggerCharacter": ",",
        "isRetrigger": true,
        "activeSignatureHelp": {
            "signatures": [{"label": "format(value: string, radix: number): string"}],
            "activeSignature": 0,
            "activeParameter": 1
        }
    });
    let params = signature_help_request_params(&uri, 8, 13, Some(context.clone()));

    assert_eq!(params["context"], context);
}
