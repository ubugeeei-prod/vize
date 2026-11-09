//! #12526: an editor session lets tsgo finish its background watch updates
//! before it closes the transport.

use super::{EditorLspSession, responder::BackgroundWatches};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt};

fn log(watches: &mut BackgroundWatches, message: &str) {
    watches.observe(&json!({"type": 3, "message": message}));
}

const CLIENT_SIDE: &str =
    "file watching: using LSP client-side watching (client supports dynamic registration)";
const ADOPTED: &str = "Adopted snapshot 1 (parent 0) as current session snapshot (replacing 0)";

#[test]
fn sessions_without_client_side_watching_are_always_settled() {
    let mut watches = BackgroundWatches::default();
    log(&mut watches, ADOPTED);
    assert!(watches.settled());
}

#[test]
fn every_adopted_snapshot_must_finish_or_fail_its_watch_update() {
    let mut watches = BackgroundWatches::default();
    log(&mut watches, CLIENT_SIDE);
    assert!(watches.settled());
    log(&mut watches, ADOPTED);
    log(
        &mut watches,
        "Adopted snapshot 2 (parent 1) as current session snapshot (replacing 1)",
    );
    log(&mut watches, "Updated watches in 15.292µs");
    log(
        &mut watches,
        "handled method 'textDocument/diagnostic' (3) in 2ms",
    );
    assert!(!watches.settled());
    log(
        &mut watches,
        "errors updating watches: [failed to register file watcher: context canceled]",
    );
    assert!(watches.settled());
}

/// A wire peer that adopts a snapshot while answering `initialize` and
/// finishes its watches 300 ms later, as tsgo's background queue does.
const PEER: &str = r#"#!/usr/bin/env node
import { appendFileSync } from "node:fs";
const logPath = LOG_PATH;
const retain = (packet) => appendFileSync(logPath, JSON.stringify(packet) + "\n");
function write(packet) {
  const payload = Buffer.from(JSON.stringify(packet));
  process.stdout.write(Buffer.concat([Buffer.from(`Content-Length: ${payload.length}\r\n\r\n`), payload]));
}
function log(message) {
  retain({ log: message });
  write({ jsonrpc: "2.0", method: "window/logMessage", params: { type: 3, message } });
}
let pending = Buffer.alloc(0);
for await (const bytes of process.stdin) {
  pending = Buffer.concat([pending, bytes]);
  for (;;) {
    const boundary = pending.indexOf("\r\n\r\n");
    if (boundary < 0) break;
    const length = Number(/content-length: *(\d+)/i.exec(pending.subarray(0, boundary).toString())[1]);
    if (pending.length < boundary + 4 + length) break;
    const message = JSON.parse(pending.subarray(boundary + 4, boundary + 4 + length).toString());
    pending = pending.subarray(boundary + 4 + length);
    if (message.method === undefined) continue;
    retain({ method: message.method });
    if (message.method === "initialize") {
      // Like a request whose snapshot adoption is logged before its response.
      log("file watching: using LSP client-side watching (client supports dynamic registration)");
      log("Adopted snapshot 1 (parent 0) as current session snapshot (replacing 0)");
      write({ jsonrpc: "2.0", id: message.id, result: { capabilities: {} } });
      setTimeout(() => log("Updated watches in 1ms"), 300);
    } else if (message.method === "shutdown") {
      write({ jsonrpc: "2.0", id: message.id, result: null });
    } else if (message.method === "exit") {
      process.exit(0);
    }
  }
}
"#;

#[test]
fn shutdown_lets_background_watch_updates_finish_before_closing() {
    let temp = tempfile::tempdir().unwrap();
    let log = temp.path().join("transcript.jsonl");
    let script = temp.path().join("peer.ts");
    let source = PEER.replace("LOG_PATH", &serde_json::to_string(&log).unwrap());
    fs::write(&script, source).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut session =
        EditorLspSession::spawn(script.to_str().unwrap(), temp.path(), temp.path()).unwrap();
    session.shutdown().unwrap();
    let transcript: Vec<Value> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        transcript,
        [
            json!({"method": "initialize"}),
            json!({"log": CLIENT_SIDE}),
            json!({"log": ADOPTED}),
            json!({"method": "initialized"}),
            json!({"log": "Updated watches in 1ms"}),
            json!({"method": "shutdown"}),
            json!({"method": "exit"}),
        ]
    );
}
