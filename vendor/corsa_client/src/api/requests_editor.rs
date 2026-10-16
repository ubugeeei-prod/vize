use crate::lsp_types::CompletionContext;
use serde::Serialize;

use super::{DocumentIdentifier, ProjectHandle, SnapshotHandle};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenameAtPositionRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub file: DocumentIdentifier,
    pub position: u32,
    pub new_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompletionAtPositionRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub file: DocumentIdentifier,
    pub position: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CompletionContext>,
}
