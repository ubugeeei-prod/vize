#![cfg(test)]
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig};
use serde_json::{Value, json};

/// Bounds a hang, not latency: the assertion is about the shutdown/exit
/// message shape, and a loaded full test run once took longer than 5 s to
/// start the fake server (v0.454.0 check run 38531768017).
const HANG_BOUND: Duration = Duration::from_secs(60);

struct RawInitialize;

impl lsp_types::request::Request for RawInitialize {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "initialize";
}

#[test]
fn graceful_close_omits_shutdown_and_exit_params() {
    if find_node().is_none() {
        eprintln!("skipping params-less shutdown regression: node is not available");
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let server = tempdir.path().join("params-less-lsp-server.mjs");
    write_fake_lsp_server(&server);

    corsa::runtime::block_on(async {
        let client = LspClient::spawn(
            LspSpawnConfig::new(&server)
                .with_request_timeout(Some(HANG_BOUND))
                .with_shutdown_timeout(HANG_BOUND),
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

fn find_node() -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|dir| {
        let candidate = dir.join("node");
        candidate.is_file().then_some(candidate)
    })
}

/// A minimal stdio LSP server that fails if `shutdown` or `exit` carries
/// `params`.
fn write_fake_lsp_server(path: &Path) {
    let source = r#"#!/usr/bin/env node
let buffer = Buffer.alloc(0);

function write(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`);
  process.stdout.write(body);
}

function handle(message) {
  const method = message.method;
  if ((method === "shutdown" || method === "exit") && "params" in message) {
    process.stderr.write(`${method} unexpectedly included params: ${JSON.stringify(message.params)}\n`);
    process.exit(1);
  }
  if (method === "initialize") {
    write({ jsonrpc: "2.0", id: message.id, result: { capabilities: {} } });
  } else if (method === "shutdown") {
    write({ jsonrpc: "2.0", id: message.id, result: null });
  } else if (method === "exit") {
    process.exit(0);
  }
}

process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const end = buffer.indexOf("\r\n\r\n");
    if (end < 0) return;
    const length = Number(/content-length:\s*(\d+)/i.exec(buffer.subarray(0, end).toString("ascii"))[1]);
    if (buffer.length < end + 4 + length) return;
    const body = buffer.subarray(end + 4, end + 4 + length).toString("utf8");
    buffer = buffer.subarray(end + 4 + length);
    handle(JSON.parse(body));
  }
});
process.stdin.on("end", () => {
  process.stderr.write("transport closed before exit\n");
  process.exit(1);
});
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
