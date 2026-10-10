//! Passive artifact controls; these are not Corsa or diagnostic correctness proof.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test-only JSON and filesystem controls use standard buffers"
)]

use std::{path::PathBuf, process::Command};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "support/lsp_process/passive.rs"]
mod passive;
use passive::Passive;

fn expected(terminal: Value) -> Value {
    json!({
        "schema":"vize-lsp-passive-terminal-evidence-v1",
        "process":{
            "pid":17,"declaredExecutable":"NONEXISTENT-PASSIVE-CONTROL",
            "arguments":["lsp"],"cwd":null,
            "sourceSha":std::env::var("SOURCE_SHA").ok(),
            "harnessPackageVersion":env!("CARGO_PKG_VERSION"),
            "identityScope":"original spawned LSP PID and declared command; no executable-byte or backend-PID join",
            "extraIdentityProbe":false
        },
        "observationTiming":"persisted after original teardown and reader joins",
        "terminalEvidence":terminal
    })
}

fn capture(root: PathBuf) -> Passive {
    let mut command = Command::new("NONEXISTENT-PASSIVE-CONTROL");
    command.arg("lsp");
    Passive::with_root(root, 17, &command)
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn passive_identity_has_no_probe_and_retains_the_entire_owned_packet() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("capture");
    let capture = capture(root.clone());
    assert_eq!(
        std::fs::read_dir(case.path()).unwrap().count(),
        0,
        "construction must not touch the filesystem"
    );
    let terminal = json!({
        "protocol":[
            {"direction":"request","message":{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}},
            {"direction":"response","message":{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///App.vue","diagnostics":[]}}}
        ],
        "stderrBytes":[112,104,97,115,101,10],
        "stderrUtf8":"phase\n",
        "exitCode":null,"exitSuccess":false,
        "stdoutReaderJoined":true,"stderrReaderJoined":true
    });
    let expected_bytes = serde_json::to_vec_pretty(&expected(terminal.clone())).unwrap();
    capture.persist(terminal).unwrap();
    assert_eq!(
        std::fs::read(root.join("evidence.json")).unwrap(),
        expected_bytes
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest,
        json!({
            "complete":true,"originalBytes":expected_bytes.len(),"persistedBytes":expected_bytes.len(),
            "limitBytes":4*1024*1024,"originalSha256":digest(&expected_bytes),"persistedSha256":digest(&expected_bytes),
            "representation":"decoded JSON messages and already-owned stderr bytes; not raw wire frames",
            "overflowMeaning":"incomplete observation, never an accepted whole packet"
        })
    );
}

#[test]
fn overflow_is_explicit_and_cannot_be_a_whole_packet() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("capture");
    let terminal = json!({"stderrUtf8":"x".repeat(5*1024*1024)});
    let entire = serde_json::to_vec_pretty(&expected(terminal.clone())).unwrap();
    capture(root.clone()).persist(terminal).unwrap();
    let prefix = std::fs::read(root.join("evidence.json")).unwrap();
    assert_eq!(prefix, entire[..4 * 1024 * 1024]);
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest,
        json!({
            "complete":false,"originalBytes":entire.len(),"persistedBytes":4*1024*1024,
            "limitBytes":4*1024*1024,"originalSha256":digest(&entire),"persistedSha256":digest(&prefix),
            "representation":"decoded JSON messages and already-owned stderr bytes; not raw wire frames",
            "overflowMeaning":"incomplete observation, never an accepted whole packet"
        })
    );
}

#[test]
fn artifact_write_failure_is_reported_without_panicking_or_overwriting() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("existing-file");
    std::fs::write(&root, b"original unrelated bytes").unwrap();
    let result = capture(root.clone()).persist(json!({"exitSuccess":false}));
    assert_eq!(
        result.as_ref().err().map(std::io::Error::kind),
        Some(std::io::ErrorKind::AlreadyExists)
    );
    assert_eq!(std::fs::read(root).unwrap(), b"original unrelated bytes");
}
