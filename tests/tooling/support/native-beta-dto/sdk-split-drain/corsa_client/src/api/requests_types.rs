use serde::Serialize;

use super::{NodeHandle, ProjectHandle, SignatureHandle, SnapshotHandle, SymbolHandle, TypeHandle};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignatureOfTypeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub r#type: TypeHandle,
    pub kind: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeLocationRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub location: NodeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrintNodeRequest {
    pub data: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub preserve_source_newlines: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub never_ascii_escape: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub terminate_unterminated_literals: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypeAssignabilityRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub source: TypeHandle,
    pub target: TypeHandle,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PropertyOfTypeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub r#type: TypeHandle,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParameterTypeRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub signature: SignatureHandle,
    pub index: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MemberInModuleExportsRequest {
    pub snapshot: SnapshotHandle,
    pub project: ProjectHandle,
    pub symbol: SymbolHandle,
    pub name: String,
}
