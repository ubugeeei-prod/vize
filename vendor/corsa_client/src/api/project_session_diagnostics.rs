use super::{
    DocumentIdentifier, FileDiagnosticsResponse, ProjectDiagnosticsResponse, ProjectSession,
    SnapshotDiagnosticsResponse,
};
use crate::Result;

impl ProjectSession {
    /// Returns diagnostics for every project in the active snapshot.
    pub async fn get_diagnostics_for_snapshot(&self) -> Result<SnapshotDiagnosticsResponse> {
        self.client()
            .get_diagnostics_for_snapshot(self.snapshot().handle.clone())
            .await
    }

    /// Returns diagnostics for every file in the active project.
    pub async fn get_diagnostics_for_project(&self) -> Result<ProjectDiagnosticsResponse> {
        self.client()
            .get_diagnostics_for_project(self.snapshot().handle.clone(), self.project().id.clone())
            .await
    }

    /// Returns diagnostics for a single file in the active project.
    pub async fn get_diagnostics_for_file(
        &self,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<FileDiagnosticsResponse> {
        self.client()
            .get_diagnostics_for_file(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
            )
            .await
    }
}
