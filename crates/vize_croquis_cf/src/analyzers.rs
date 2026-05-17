//! Individual cross-file analyzers.
//!
//! Each analyzer focuses on a specific aspect of cross-file analysis.
//!
//! ## Suppression Directives
//!
//! Use `// @vize forget` comment to suppress specific warnings:
//! ```typescript
//! // @vize forget
//! const { count } = inject('state')  // No warning for destructuring
//! ```

mod boundary;
mod component_resolution;
mod cross_file_reactivity;
mod element_id;
mod emit;
mod event_bubbling;
mod fallthrough;
mod props_validation;
mod provide_inject;
mod race_conditions;
mod reactivity;
mod setup_context;

// Re-export analyzer types
pub use boundary::{BoundaryInfo, BoundaryKind, analyze_boundaries};
pub use component_resolution::{ComponentResolutionIssue, analyze_component_resolution};
pub use element_id::{UniqueIdIssue, analyze_element_ids};
pub use emit::{EmitFlow, analyze_emits};
pub use event_bubbling::{EventBubble, analyze_event_bubbling};
pub use fallthrough::{FallthroughInfo, analyze_fallthrough};
pub use props_validation::{PropsValidationIssue, analyze_props_validation};
pub(crate) use provide_inject::{
    ProvideInjectIndex, analyze_provide_inject_with_index, build_provide_inject_tree_with_index,
};
pub use provide_inject::{ProvideInjectMatch, ProvideInjectTree};
pub use race_conditions::RaceConditionIssue;
pub(crate) use race_conditions::analyze_race_conditions_with_index;
pub use reactivity::{ReactivityIssue, ReactivityIssueKind, analyze_reactivity};

// Cross-file reactivity tracking
#[cfg(test)]
pub(crate) use cross_file_reactivity::CrossFileReactivityIssueKind;
pub use cross_file_reactivity::{CrossFileReactivityIssue, analyze_cross_file_reactivity};

// Setup context violation tracking
pub use setup_context::{SetupContextIssue, analyze_setup_context};
