//! Advance an existing Open only after this foreground request really opened it.

use std::sync::atomic::AtomicBool;
use std::time::Instant;
use tower_lsp::lsp_types::Url;

use super::{DiagnosticsJobCause, InitialDiagnosticsScheduler};
use crate::server::state::{CorsaRequestStamp, ServerState};

pub(in crate::server) struct ForegroundOpen {
    uri: Url,
    version: i32,
    sequence: u64,
    prepared: AtomicBool,
}

impl ForegroundOpen {
    pub(in crate::server) fn preparation(&self) -> (&Url, &AtomicBool) {
        (&self.uri, &self.prepared)
    }
}

impl InitialDiagnosticsScheduler {
    pub(in crate::server) fn foreground_open(
        &self,
        uri: &Url,
        state: &ServerState,
    ) -> Option<ForegroundOpen> {
        self.sender.as_ref()?;
        if !state.is_lsp_typecheck_enabled() || state.project_context_retired() {
            return None;
        }
        let version = state.documents.version(uri)?;
        let pending = self.pending.lock();
        let job = pending.jobs.get(uri)?;
        (job.cause == DiagnosticsJobCause::Open && job.version == version).then(|| ForegroundOpen {
            uri: uri.clone(),
            version,
            sequence: job.sequence,
            prepared: AtomicBool::new(false),
        })
    }

    /// Called after the final native scope is released and authored_response passed.
    pub(in crate::server) fn foreground_ready(
        &self,
        ticket: &ForegroundOpen,
        stamp: CorsaRequestStamp,
        prepared: bool,
        state: &ServerState,
        now: Instant,
    ) -> bool {
        if !prepared {
            return false;
        }
        let advanced = self.pending.lock().advance_foreground_open(
            &ticket.uri,
            ticket.version,
            ticket.sequence,
            stamp,
            state,
            now,
        );
        if advanced && let Some(sender) = &self.sender {
            let _ = sender.try_send(());
        }
        advanced
    }
}

#[cfg(test)]
mod tests;
