//! Owned cross-file aggregation produced from peer Croquis semantic products.

use vize_atlas::{Product, SourceId, SourceRevision};
use vize_carton::CompactString;
use vize_croquis::SemanticSourceRange;

/// Stable project-level view over the semantic products selected by one query.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CroquisProjectSnapshot {
    /// Source used as the project-query anchor.
    pub anchor: Option<SourceId>,
    /// Deterministically ordered source summaries.
    pub sources: Vec<CroquisProjectSource>,
    /// Component usages resolved against all source names in the snapshot.
    pub component_usages: Vec<CroquisProjectComponentUsage>,
    /// Provide/inject keys grouped across source boundaries.
    pub injection_groups: Vec<CroquisProjectInjectionGroup>,
}

/// Summary of one cached Croquis semantic product in a project snapshot.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CroquisProjectSource {
    pub source: SourceId,
    pub revision: SourceRevision,
    pub name: CompactString,
    pub binding_count: usize,
    pub scope_count: usize,
    pub template_expression_count: usize,
    pub component_usage_count: usize,
}

/// One component usage and every project source that can satisfy its name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CroquisProjectComponentUsage {
    pub source: SourceId,
    pub name: CompactString,
    pub range: SemanticSourceRange,
    pub candidates: Vec<SourceId>,
}

/// Providers and consumers sharing one normalized injection key.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CroquisProjectInjectionGroup {
    pub key: CompactString,
    pub providers: Vec<SourceId>,
    pub consumers: Vec<SourceId>,
}

/// Atlas identity for opt-in cross-file Croquis aggregation.
pub struct CroquisProjectProduct;

impl Product for CroquisProjectProduct {
    type Value = CroquisProjectSnapshot;

    const NAME: &'static str = "croquis.project";
}
