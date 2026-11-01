//! Split an initial read from foreground state and return an inert result.
#![expect(
    clippy::disallowed_types,
    reason = "owned SDK and mirror leases cross workers"
)]

use std::{sync::Arc, time::Duration};

use super::{CorsaBridge, CorsaBridgeError};
use crate::{
    corsa_bridge::{EditorMirrorSession, LspDiagnostic, worker::WorkerError},
    lsp_client::InitialDiagnosticsRead,
};

pub struct InitialApiDiagnostics {
    pub(super) read: Arc<InitialDiagnosticsRead>,
    // Keep physical mirror storage alive until the entered RPC drains. This
    // does not override explicit shutdown/clear or make mutable files immutable.
    pub(super) mirror: Arc<EditorMirrorSession>,
}

pub struct PendingInitialApiDiagnostics {
    pub(super) read: Arc<InitialDiagnosticsRead>,
    pub(super) result: Result<Option<Vec<lsp_types::Diagnostic>>, CorsaBridgeError>,
}

pub enum InitialApiDiagnosticsOutcome {
    Stale,
    Unsupported,
    Complete(Vec<LspDiagnostic>),
    Failed(CorsaBridgeError),
}

impl CorsaBridge {
    /// Only this read-only lane owns the lease during synchronous native IPC.
    /// Cancelling its future leaves an entered request draining on that lane.
    pub async fn read_initial_api_diagnostics(
        &self,
        request: InitialApiDiagnostics,
    ) -> PendingInitialApiDiagnostics {
        let read = Arc::clone(&request.read);
        let result = self
            .initial_diagnostics_worker
            .submit_async(
                Duration::from_millis(self.config.timeout_ms.max(1)),
                move |_| {
                    let _mirror = request.mirror;
                    request
                        .read
                        .run()
                        .map_err(CorsaBridgeError::CommunicationError)
                },
            )
            .await;
        let result = match result {
            Ok(result) => result,
            Err(WorkerError::TimedOut) => Err(CorsaBridgeError::Timeout),
            Err(WorkerError::Stopped) => Err(CorsaBridgeError::ProcessTerminated),
        };
        PendingInitialApiDiagnostics { read, result }
    }
}

#[cfg(all(test, unix))]
mod tests;
