//! #12526: an editor session lets tsgo finish its background watch updates
//! before it closes the transport.

use super::{EditorLspSession, responder::BackgroundWatches};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

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

/// A wire peer that adopts a snapshot while answering `initialize`. With
/// `FINISHES`, it then registers a watch, as tsgo's background queue does, and
/// reports the update once the client answers. Otherwise it never does.
const PEER: &str = include_str!("../../../tests/programs/record-watch-settlement.js.in");

/// vscode-languageclient's default `stop` timeout, for shutdown and exit.
const CLIENT_STOP_BUDGET: Duration = Duration::from_secs(2);

/// Spawn the peer, shut the session down, and return its transcript and how
/// long `shutdown` took.
fn shut_down(finishes: bool) -> (Vec<Value>, Duration) {
    let temp = tempfile::tempdir().unwrap();
    let log = temp.path().join("transcript.jsonl");
    let script = temp.path().join("peer.ts");
    let source = PEER
        .replace("LOG_PATH", &serde_json::to_string(&log).unwrap())
        .replace("FINISHES", if finishes { "true" } else { "false" });
    fs::write(&script, source).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut session =
        EditorLspSession::spawn(script.to_str().unwrap(), temp.path(), temp.path()).unwrap();
    let started = Instant::now();
    session.shutdown().unwrap();
    let elapsed = started.elapsed();
    let transcript = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (transcript, elapsed)
}

#[test]
fn shutdown_lets_background_watch_updates_finish_before_closing() {
    let (transcript, _) = shut_down(true);
    assert_eq!(
        transcript,
        [
            json!({"method": "initialize"}),
            json!({"log": CLIENT_SIDE}),
            json!({"log": ADOPTED}),
            json!({"method": "initialized"}),
            json!({"answered": "watch-1"}),
            json!({"log": "Updated watches in 1ms"}),
            json!({"method": "shutdown"}),
            json!({"method": "exit"}),
        ]
    );
}

#[test]
fn an_unfinished_watch_update_never_holds_shutdown_past_the_client_budget() {
    let (transcript, elapsed) = shut_down(false);
    assert_eq!(
        transcript,
        [
            json!({"method": "initialize"}),
            json!({"log": CLIENT_SIDE}),
            json!({"log": ADOPTED}),
            json!({"method": "initialized"}),
            json!({"method": "shutdown"}),
            json!({"method": "exit"}),
        ]
    );
    // The editor's whole stop shares this budget with tsgo's own shutdown.
    assert!(
        elapsed < CLIENT_STOP_BUDGET / 2,
        "shutdown took {elapsed:?} of the client's {CLIENT_STOP_BUDGET:?} stop budget"
    );
}
