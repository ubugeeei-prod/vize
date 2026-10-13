use serde::Serialize;

use super::{
    DocumentIdentifier, FileChanges, NodeHandle, OverlayChanges, ProjectHandle, SnapshotHandle,
    SymbolHandle, TypeHandle,
};

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateSnapshotRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_projects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_changes: Option<FileChanges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_changes: Option<OverlayChanges>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParseConfigFileRequest {
    pub file: DocumentIdentifier,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReleaseRequest<'a> {
    pub handle: &'a str,
    pub snapshot: &'a SnapshotHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotFileRequest {
    pub snapshot: SnapshotHandle,
    pub file: DocumentIdentifier,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotProjectRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotProjectFileRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub file: DocumentIdentifier,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResolveNameRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<NodeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<DocumentIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
    pub meaning: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_globals: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SymbolAtPositionRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub file: DocumentIdentifier,
    pub position: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SymbolAtLocationRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub location: NodeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShorthandValueRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub location: NodeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeOfSymbolAtLocationRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub symbol: SymbolHandle,
    pub location: NodeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IntrinsicTypeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeNodeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub r#type: TypeHandle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<NodeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::UpdateSnapshotRequest;

    #[test]
    fn update_snapshot_request_uses_current_open_projects_wire_field() {
        let payload = serde_json::to_value(UpdateSnapshotRequest {
            open_projects: vec!["/workspace/tsconfig.json".into()],
            file_changes: None,
            overlay_changes: None,
        })
        .unwrap();

        assert_eq!(
            payload,
            json!({
                "openProjects": ["/workspace/tsconfig.json"],
            })
        );
    }
}
