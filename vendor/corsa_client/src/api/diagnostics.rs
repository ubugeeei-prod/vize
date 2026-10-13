use crate::lsp_types::Diagnostic;
use serde::{Deserialize, Serialize};

use super::{DocumentIdentifier, ProjectHandle, SnapshotHandle};

/// Diagnostics for a single file grouped by TypeScript category.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiagnosticsResponse {
    /// File path or URI the diagnostics belong to.
    pub file: DocumentIdentifier,
    /// Parse and grammar diagnostics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub syntactic: Vec<Diagnostic>,
    /// Checker and semantic diagnostics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic: Vec<Diagnostic>,
    /// Suggestion diagnostics such as unused or style-oriented hints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestion: Vec<Diagnostic>,
}

/// Diagnostics for every file in a project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDiagnosticsResponse {
    /// Project handle the diagnostics were collected from.
    pub project: ProjectHandle,
    /// Per-file diagnostics inside the project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FileDiagnosticsResponse>,
}

/// Diagnostics for every project in a snapshot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiagnosticsResponse {
    /// Snapshot handle the diagnostics were collected from.
    pub snapshot: SnapshotHandle,
    /// Per-project diagnostics in the snapshot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectDiagnosticsResponse>,
}
