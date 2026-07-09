//! Individual cross-file rules.
//!
//! Each rule focuses on a specific aspect of cross-file analysis.
//!
//! ## Suppression Directives
//!
//! Use `// @vize forget` comment to suppress specific warnings:
//! ```typescript
//! // @vize forget
//! const { count } = inject('state')  // No warning for destructuring
//! ```

mod boundary;
mod complexity;
#[cfg(test)]
mod complexity_hotspots_tests;
#[cfg(test)]
mod complexity_injection_tests;
#[cfg(test)]
mod complexity_tests;
mod component_resolution;
pub(crate) mod cross_file_reactivity;
mod element_id;
mod emit;
mod event_bubbling;
mod fallthrough;
mod props_validation;
mod provide_inject;
mod race_conditions;
mod reactivity;
mod setup_context;

// Re-export rule result types
pub use boundary::{BoundaryInfo, BoundaryKind, analyze_boundaries};
pub use complexity::{
    ComplexityBand, ComplexityDimension, ComplexityDimensionBreakdown, ComplexityDimensionScores,
    ComplexityHotspot, ComplexityInput, ComplexityReport, band_for_score,
    summarize_complexity_hotspots, summarize_complexity_with_graph,
};
pub use component_resolution::{ComponentResolutionIssue, analyze_component_resolution};
pub use element_id::{UniqueIdIssue, analyze_element_ids};
pub use emit::{EmitFlow, analyze_emits};
pub use event_bubbling::{EventBubble, analyze_event_bubbling};
pub use fallthrough::{
    FallthroughComponentFact, FallthroughInfo, FallthroughSummary, FallthroughUsageAttrFact,
    FallthroughUsageAttrKind, FallthroughUsageFact, analyze_fallthrough,
    collect_fallthrough_component_facts, collect_fallthrough_usage_facts, summarize_fallthrough,
};
pub use props_validation::{
    PropsValidationIssue, PropsValidationIssueKind, analyze_props_validation,
};
pub(crate) use provide_inject::{
    ProvideInjectIndex, analyze_provide_inject_with_index, build_provide_inject_tree_with_index,
};
pub use provide_inject::{ProvideInjectMatch, ProvideInjectTree, ProvideInjectTreeSummary};
pub use race_conditions::RaceConditionIssue;
pub(crate) use race_conditions::analyze_race_conditions_with_index;
pub use reactivity::{ReactivityIssue, ReactivityIssueKind, analyze_reactivity};

// Cross-file reactivity tracking
#[cfg(test)]
pub(crate) use cross_file_reactivity::CrossFileReactivityIssueKind;
pub use cross_file_reactivity::{CrossFileReactivityIssue, analyze_cross_file_reactivity};

// Setup context violation tracking
pub use setup_context::{SetupContextIssue, analyze_setup_context};
