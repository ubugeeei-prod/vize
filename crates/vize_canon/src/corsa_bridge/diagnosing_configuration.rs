//! Actual configuration observations from the same process returning diagnostics.

use corsa::api::{ProjectResponse, UpdateSnapshotResponse};
use corsa_lsp::InitializeApiSessionResult;

/// Complete API observations before and after one native diagnostic request.
///
/// These are historical receipts from the diagnosing LSP's project session.
/// Their handles have been released; they are not live query handles or a proof
/// that the intervening diagnostic used an identical source/configuration graph.
pub struct DiagnosingConfiguration {
    pub(crate) session: InitializeApiSessionResult,
    pub(crate) before: DiagnosingSnapshot,
    pub(crate) after: DiagnosingSnapshot,
}

impl DiagnosingConfiguration {
    #[must_use]
    pub fn session(&self) -> &InitializeApiSessionResult {
        &self.session
    }
    #[must_use]
    pub fn before(&self) -> &DiagnosingSnapshot {
        &self.before
    }
    #[must_use]
    pub fn after(&self) -> &DiagnosingSnapshot {
        &self.after
    }
}

/// An unmodified snapshot response and the actual project's full response.
pub struct DiagnosingSnapshot {
    pub(crate) response: UpdateSnapshotResponse,
    pub(crate) project: ProjectResponse,
}

impl DiagnosingSnapshot {
    #[must_use]
    pub fn response(&self) -> &UpdateSnapshotResponse {
        &self.response
    }
    #[must_use]
    pub fn project(&self) -> &ProjectResponse {
        &self.project
    }
}
