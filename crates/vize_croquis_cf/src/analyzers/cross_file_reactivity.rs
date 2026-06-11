//! Cross-File Reactivity Tracking.
//!
//! NOTE: This module is under active development. Many items are reserved
//! for future cross-file analysis features.
#![allow(unused)]
//!
//! Tracks reactive values across module boundaries, including:
//! - Composable exports/imports
//! - Provide/inject chains
//! - Props passed between components
//! - Pinia store usage across components
//!
//! ## Design
//!
//! This analyzer builds a "Reactivity Flow Graph" that tracks how reactive
//! values flow between files. It detects when reactivity is accidentally
//! broken at module boundaries.
//!
//! ```text
//! useCounter.ts ──export──> Component.vue
//!      │                         │
//!      └── ref(0) ───────────> const { count } = useCounter()
//!                                    ↑
//!                               REACTIVITY LOST!
//! ```

mod analyzer;
mod collect;
mod diagnostics;
mod flow_composable;
mod flow_core;
mod flow_props;
mod flow_provide_inject;
mod issues;
mod name_helpers;
mod path_helpers;
mod prop_helpers;
mod provide_helpers;
pub(crate) mod store_detection;
mod types;

pub use analyzer::CrossFileReactivityAnalyzer;
pub use types::{
    CrossFileReactiveValue, CrossFileReactivityIssue, CrossFileReactivityIssueKind,
    ReactiveConsumption, ReactiveExposure, ReactiveValueId, ReactivityFlow, ReactivityFlowKind,
    ReactivityLossReason,
};

use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::ModuleRegistry;

// Re-export types used in tests (brought in via `super::*`).
pub(crate) use crate::diagnostics::DiagnosticSeverity;
pub(crate) use crate::registry::FileId;
pub(crate) use vize_carton::CompactString;

/// Public API: Analyze cross-file reactivity.
pub fn analyze_cross_file_reactivity(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<CrossFileReactivityIssue>, Vec<CrossFileDiagnostic>) {
    let analyzer = CrossFileReactivityAnalyzer::new(registry, graph);
    analyzer.analyze()
}

#[cfg(test)]
#[path = "cross_file_reactivity_tests.rs"]
mod tests;
