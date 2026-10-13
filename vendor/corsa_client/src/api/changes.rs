use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{DocumentIdentifier, ProjectHandle, SnapshotHandle};

/// Per-file snapshot delta grouped by operation kind.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeSummary {
    /// Files whose contents changed but whose identity stayed the same.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<DocumentIdentifier>,
    /// Newly created files that should enter the snapshot graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created: Vec<DocumentIdentifier>,
    /// Deleted files that should be removed from the snapshot graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<DocumentIdentifier>,
}

/// Change payload accepted by `updateSnapshot`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FileChanges {
    /// Apply an incremental file-by-file delta.
    Summary(FileChangeSummary),
    #[serde(rename_all = "camelCase")]
    /// Discard all prior file state and force a full invalidation.
    InvalidateAll {
        /// When `true`, the server invalidates all tracked file state.
        invalidate_all: bool,
    },
}

/// In-memory document update applied through `updateSnapshot`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayUpdate {
    /// File path or URI identifying the virtual document.
    pub document: DocumentIdentifier,
    /// Full in-memory document contents.
    pub text: String,
    /// Optional monotonic version associated with the overlay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    /// Optional language identifier such as `typescript` or `vue`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_id: Option<String>,
}

/// Overlay delta accepted by `updateSnapshot`.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayChanges {
    /// Overlay documents whose full contents should be inserted or replaced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upsert: Vec<OverlayUpdate>,
    /// Overlay documents that should be removed from the active snapshot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delete: Vec<DocumentIdentifier>,
}

/// Parameters passed to [`ApiClient::update_snapshot`](crate::ApiClient::update_snapshot).
///
/// # Examples
///
/// ```
/// use corsa_client::{DocumentIdentifier, FileChangeSummary, FileChanges, UpdateSnapshotParams};
///
/// let params = UpdateSnapshotParams {
///     open_project: Some("/workspace/tsconfig.json".into()),
///     file_changes: Some(FileChanges::Summary(FileChangeSummary {
///         changed: vec![DocumentIdentifier::from("/workspace/main.ts")],
///         created: Vec::new(),
///         deleted: Vec::new(),
///     })),
///     overlay_changes: None,
/// };
///
/// assert_eq!(params.open_project.as_deref(), Some("/workspace/tsconfig.json"));
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshotParams {
    /// Preferred project to open eagerly, usually a `tsconfig` path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_project: Option<String>,
    /// Incremental file invalidation payload, or `None` for a no-op refresh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_changes: Option<FileChanges>,
    /// In-memory document updates applied on top of on-disk state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_changes: Option<OverlayChanges>,
}

/// Snapshot changes scoped to a single project.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileChanges {
    /// Files inside the project that changed contents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<String>,
    /// Files inside the project that were removed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted_files: Vec<String>,
}

/// Project-level delta information returned by Corsa.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotChanges {
    /// Project-scoped change details keyed by project handle.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub changed_projects: BTreeMap<ProjectHandle, ProjectFileChanges>,
    /// Projects removed from the snapshot as part of the update.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_projects: Vec<ProjectHandle>,
}

/// Raw response returned by `updateSnapshot`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshotResponse {
    /// Newly created or refreshed snapshot handle.
    pub snapshot: SnapshotHandle,
    /// Projects currently known inside the snapshot.
    pub projects: Vec<super::ProjectResponse>,
    /// Project-level delta information when the server computed it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<SnapshotChanges>,
}
