use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig};
use serde_json::{Value, json};

struct RawInitialize;

impl lsp_types::request::Request for RawInitialize {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "initialize";
}

#[test]
fn graceful_close_omits_shutdown_and_exit_params() {
    if find_python3().is_none() {
        eprintln!("skipping params-less shutdown regression: python3 is not available");
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let server = tempdir.path().join("params_less_lsp_server.py");
    write_fake_lsp_server(&server);

    corsa::runtime::block_on(async {
        let client = LspClient::spawn(
            LspSpawnConfig::new(&server)
                .with_request_timeout(Some(Duration::from_secs(5)))
                .with_shutdown_timeout(Duration::from_secs(5)),
        )
        .await
        .unwrap();

        let initialize = client
            .request::<RawInitialize>(json!({
                "processId": std::process::id(),
                "rootUri": null,
                "capabilities": {}
            }))
            .await
            .unwrap();
        assert!(initialize["capabilities"].is_object(), "{initialize:#}");

        client.graceful_close().await.unwrap();
    });
}

fn find_python3() -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|dir| {
        let candidate = dir.join("python3");
        candidate.is_file().then_some(candidate)
    })
}

fn write_fake_lsp_server(path: &Path) {
    let source = r#"#!/usr/bin/env python3
import json
import sys


def read_message():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if line in (b"\r\n", b"\n"):
            break
        name, value = line.decode("ascii").split(":", 1)
        headers[name.lower()] = value.strip()
    length = int(headers["content-length"])
    return json.loads(sys.stdin.buffer.read(length))


def write_message(message):
    body = json.dumps(message, separators=(",", ":")).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
    sys.stdout.buffer.write(body)
    sys.stdout.buffer.flush()


while True:
    message = read_message()
    if message is None:
        sys.exit("transport closed before exit")
    method = message.get("method")
    if method in ("shutdown", "exit") and "params" in message:
        sys.exit(f"{method} unexpectedly included params: {message['params']!r}")
    if method == "initialize":
        write_message({
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {"capabilities": {}},
        })
    elif method == "shutdown":
        write_message({"jsonrpc": "2.0", "id": message["id"], "result": None})
    elif method == "exit":
        sys.exit(0)
"#;
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(source.trim_start().as_bytes()).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}
