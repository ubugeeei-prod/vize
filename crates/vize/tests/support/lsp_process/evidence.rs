//! Opt-in, passive custody of the messages and stderr already owned by a test.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "integration evidence retains complete standard JSON and UTF-8 buffers"
)]

use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Value, json};
use vize_l0::cstr;

use super::LspProcess;

#[derive(Clone, Default)]
pub(super) struct Evidence(Arc<OnceLock<Mutex<Vec<Value>>>>);

impl Evidence {
    pub(super) fn record(&self, direction: &str, message: &Value) {
        if let Some(evidence) = self.0.get() {
            evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(json!({ "direction": direction, "message": message }));
        }
    }
}

impl LspProcess {
    /// Enable recording before the first original request, without any probe.
    pub fn retain_protocol(&mut self) {
        self.evidence.0.get_or_init(|| Mutex::new(Vec::new()));
    }

    /// Read only after the original graceful exit has joined both readers.
    pub fn terminal_evidence(&self) -> Value {
        let stderr = self
            .stderr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let protocol = self.evidence.0.get().map(|evidence| {
            evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        json!({
            "pid": self.child.as_ref().map(std::process::Child::id),
            "executable": env!("CARGO_BIN_EXE_vize"),
            "sourceSha": std::env::var("SOURCE_SHA").ok(),
            "harnessPackageVersion": env!("CARGO_PKG_VERSION"),
            "protocolRepresentation": "complete decoded JSON messages, not raw wire frames",
            "protocol": protocol.as_deref(),
            "publishedDiagnostics": self.published_diagnostics(),
            "stderrBytes": &*stderr,
            "stderrUtf8": std::string::String::from_utf8_lossy(&stderr),
            "exitStatus": self.status.map(|status| cstr!("{status}")),
            "exitSuccess": self.status.map(|status| status.success()),
            "exitCode": self.status.and_then(|status| status.code()),
            "stdoutReaderJoined": self.stdout_joined,
            "stderrReaderJoined": self.stderr_joined
        })
    }
}
