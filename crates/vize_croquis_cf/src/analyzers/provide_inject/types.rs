use crate::registry::FileId;
use vize_carton::CompactString;

#[derive(Debug, Clone)]
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

/// Tree representation of provide/inject relationships.
#[derive(Debug, Clone)]
pub struct ProvideInjectTree {
    /// Root nodes (components that provide but don't inject from ancestors).
    pub roots: Vec<ProvideNode>,
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct ProvideInfo {
    /// The provide key.
    pub key: CompactString,
    /// The provided type (if available).
    pub value_type: Option<CompactString>,
    /// Source offset.
    pub offset: u32,
    /// Number of consumers.
    pub consumer_count: usize,
}

/// Information about an inject call.
#[derive(Debug, Clone)]
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
