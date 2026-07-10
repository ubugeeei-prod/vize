//! Cross-file analyzer state and module wiring.

mod accessors;
mod constructors;
mod deps;
mod diagnostics;
mod files;
mod paths;
mod run;
mod single_file;

use super::types::CrossFileOptions;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::FxHashMap;
use vize_croquis::{AnalyzerOptions, EffectGraphSummary};

/// Cross-file analyzer for Vue projects.
pub struct CrossFileAnalyzer {
    /// Analysis options.
    options: CrossFileOptions,
    /// Module registry.
    registry: ModuleRegistry,
    /// Dependency graph.
    graph: DependencyGraph,
    /// Parser-built local reactive effect summaries by file.
    effect_graph_summaries: FxHashMap<FileId, EffectGraphSummary>,
    /// Single-file analyzer options.
    single_file_options: AnalyzerOptions,
}

impl Default for CrossFileAnalyzer {
    fn default() -> Self {
        Self::new(CrossFileOptions::default())
    }
}
