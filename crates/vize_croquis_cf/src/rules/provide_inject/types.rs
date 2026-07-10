use crate::registry::FileId;
use serde::Serialize;
use vize_carton::CompactString;

/// A unique provider-call/consumer-inject relationship.
///
/// When the same relationship occurs through multiple render branches, `path`
/// is the deterministic shortest representative. Tree construction retains all
/// branch paths separately.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideInjectMatch {
    /// Component providing the value.
    pub provider: FileId,
    /// Component injecting the value.
    pub consumer: FileId,
    /// The provide/inject key.
    pub key: CompactString,
    /// Stable key identity including string/symbol namespace.
    pub key_identity: CompactString,
    /// Path from provider to consumer.
    pub path: Vec<FileId>,
    /// Whether types match (if available).
    pub type_match: Option<bool>,
    /// Provider offset in source.
    pub provide_offset: u32,
    /// Consumer offset in source.
    pub inject_offset: u32,
}

/// One rendered ancestor branch for an inject call.
///
/// Unlike [`ProvideInjectMatch`], this also records branches that terminate
/// without finding a provider so diagnostics and tree output can retain them.
#[derive(Debug, Clone)]
pub(crate) struct ProvideInjectBranch {
    pub consumer: FileId,
    pub key_identity: CompactString,
    pub path: Vec<FileId>,
    pub provider: Option<FileId>,
    pub provide_offset: Option<u32>,
    pub inject_offset: u32,
}

/// Tree representation of provide/inject relationships.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideInjectTree {
    /// Natural roots plus deterministic branch roots for cyclic components.
    pub roots: Vec<ProvideNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideNode {
    /// File ID of this component.
    pub file_id: FileId,
    /// Component name (if available).
    pub component_name: Option<CompactString>,
    /// Keys provided by this component.
    pub provides: Vec<ProvideInfo>,
    /// Keys injected by this component.
    pub injects: Vec<InjectInfo>,
    /// Children components that inject from this provider.
    pub children: Vec<ProvideNode>,
}

/// Information about a provide call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideInfo {
    /// The provide key.
    pub key: CompactString,
    /// The provided type (if available).
    pub value_type: Option<CompactString>,
    /// Source offset.
    pub offset: u32,
    /// Number of rendered consumer branch occurrences.
    pub consumer_count: usize,
}

/// Information about an inject call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectInfo {
    /// The inject key.
    pub key: CompactString,
    /// Whether a default value is provided.
    pub has_default: bool,
    /// The provider file (if found).
    pub provider: Option<FileId>,
    /// Source offset.
    pub offset: u32,
}
