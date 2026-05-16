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
mod diagnostics;
mod types;

pub use analyzer::CrossFileReactivityAnalyzer;
pub use types::{
    CrossFileReactiveValue, CrossFileReactivityIssue, CrossFileReactivityIssueKind,
    ReactiveConsumption, ReactiveExposure, ReactiveValueId, ReactivityFlow, ReactivityFlowKind,
    ReactivityLossReason,
};

use crate::cross_file::diagnostics::CrossFileDiagnostic;
use crate::cross_file::graph::DependencyGraph;
use crate::cross_file::registry::ModuleRegistry;

// Re-export types used in tests (brought in via `super::*`).
pub(crate) use crate::cross_file::diagnostics::DiagnosticSeverity;
pub(crate) use crate::cross_file::registry::FileId;
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
#[path = "../cross_file_reactivity_tests.rs"]
mod tests;
