//! Opt-in persistence after the original process teardown, without identity probes.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "test-only passive custody uses standard files and JSON buffers"
)]

use std::{
    io,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MAX_PERSISTED_BYTES: usize = 4 * 1024 * 1024;

#[path = "process_state.rs"]
mod process_state;

pub(super) struct Passive {
    root: PathBuf,
    identity: Value,
    retain_terminal: bool,
}

impl Passive {
    pub(super) fn new(pid: u32, command: &Command) -> Option<Self> {
        let (root, retain_terminal) = std::env::var_os("VIZE_LSP_PASSIVE_EVIDENCE")
            .map(|root| (root, true))
            .or_else(|| {
                std::env::var_os("VIZE_LSP_EXIT_FAILURE_EVIDENCE").map(|root| (root, false))
            })?;
        let root = PathBuf::from(root).join(format!("process-{pid}"));
        Some(Self::with_root(root, pid, command, retain_terminal))
    }

    pub(super) fn with_root(
        root: PathBuf,
        pid: u32,
        command: &Command,
        retain_terminal: bool,
    ) -> Self {
        // Read only already-available command metadata. No filesystem access,
        // hashing, process probe or extra request precedes original initialize.
        Self {
            root,
            retain_terminal,
            identity: json!({
                "pid":pid,
                "declaredExecutable":command.get_program().to_string_lossy(),
                "arguments":command.get_args().map(|arg|arg.to_string_lossy()).collect::<Vec<_>>(),
                "cwd":command.get_current_dir(),
                "sourceSha":std::env::var("SOURCE_SHA").ok(),
                "harnessPackageVersion":env!("CARGO_PKG_VERSION"),
                "identityScope":"original spawned LSP PID and declared command; no executable-byte or backend-PID join",
                "extraIdentityProbe":false
            }),
        }
    }

    pub(super) fn retains_terminal(&self) -> bool {
        self.retain_terminal
    }

    /// The original deadline has already failed. Observe before the caller's
    /// unchanged kill/reap, then retain the terminal packet after reader joins.
    pub(super) fn record_exit_deadline(
        &mut self,
        pid: u32,
        timeout: Duration,
        stdin_open: bool,
        owned_evidence: Value,
    ) {
        self.retain_terminal = true;
        let observed_at = unix_millis();
        let state = process_state::snapshot(pid);
        let result = self.persist_packet("exit-deadline.json", "exit-deadline-manifest.json", json!({
            "schema":"vize-lsp-exit-deadline-evidence-v1", "process":self.identity,
            "originalTimeoutMillis":timeout.as_millis(), "stdinOpen":stdin_open,
            "observedAtUnixMillis":observed_at,"finishedAtUnixMillis":unix_millis(),
            "observationTiming":"after original exit deadline, before original kill; non-atomic observations",
            "stderrCompleteness":"reader owns read_to_end bytes until EOF; final stderr follows original reader joins in evidence.json",
            "ownedEvidenceAtDeadline":owned_evidence,"processState":state,
            "llvmProfileFile":std::env::var_os("LLVM_PROFILE_FILE").map(|value|value.to_string_lossy().into_owned())
        }));
        report_error(result);
    }

    pub(super) fn persist_reporting_error(&self, terminal_evidence: Value) {
        report_error(self.persist(terminal_evidence));
    }

    pub(super) fn persist(&self, terminal_evidence: Value) -> io::Result<()> {
        if !self.retain_terminal {
            return Ok(());
        }
        self.persist_packet(
            "evidence.json",
            "manifest.json",
            json!({
                "schema":"vize-lsp-passive-terminal-evidence-v1",
                "process":self.identity,
                "observationTiming":"persisted after original teardown and reader joins",
                "terminalEvidence":terminal_evidence
            }),
        )
    }

    fn persist_packet(&self, file: &str, manifest: &str, packet: Value) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(&packet)?;
        // The original helper's in-memory buffers remain unchanged. Bound the
        // persisted artifact and mark overflow explicitly; a prefix is not a
        // complete packet and cannot qualify a successful diagnostic capture.
        let stored = bytes.len().min(MAX_PERSISTED_BYTES);
        let complete = stored == bytes.len();
        std::fs::create_dir_all(&self.root)?;
        std::fs::write(self.root.join(file), &bytes[..stored])?;
        std::fs::write(
            self.root.join(manifest),
            serde_json::to_vec_pretty(&json!({
                "complete":complete,
                "originalBytes":bytes.len(),
                "persistedBytes":stored,
                "limitBytes":MAX_PERSISTED_BYTES,
                "originalSha256":hex(&Sha256::digest(&bytes)),
                "persistedSha256":hex(&Sha256::digest(&bytes[..stored])),
                "representation":"decoded JSON messages and already-owned stderr bytes; not raw wire frames",
                "overflowMeaning":"incomplete observation, never an accepted whole packet"
            }))?,
        )
    }
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn report_error(result: io::Result<()>) {
    if let Err(error) = result {
        // Observation errors must not replace an original assertion panic.
        let _ = writeln!(
            std::io::stderr().lock(),
            "passive LSP evidence could not be persisted: {error}"
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[usize::from(byte >> 4)] as char);
        result.push(HEX[usize::from(byte & 15)] as char);
    }
    result
}
