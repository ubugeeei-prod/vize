use serde::Serialize;

use super::{NodeHandle, ProjectHandle, SignatureHandle, SnapshotHandle, SymbolHandle, TypeHandle};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SymbolBatchRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub symbols: Vec<SymbolHandle>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeBatchRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub locations: Vec<NodeHandle>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PositionBatchRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub file: super::DocumentIdentifier,
    pub positions: Vec<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SymbolOnlyRequest {
    pub snapshot: SnapshotHandle,
    pub symbol: SymbolHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SymbolProjectRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub symbol: SymbolHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeOnlyRequest {
    pub snapshot: SnapshotHandle,
    pub r#type: TypeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignatureOnlyRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub signature: SignatureHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeProjectRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub r#type: TypeHandle,
}
