//! Core state for cross-file reactivity tracking.

use super::types::{CrossFileReactivityIssue, ProvideDefinition, ReactivityFlow};
use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::FxHashMap;

pub struct CrossFileReactivityAnalyzer<'a> {
    pub(super) registry: &'a ModuleRegistry,
    pub(super) graph: &'a DependencyGraph,
    /// Reactivity flows between files.
    pub(super) flows: Vec<ReactivityFlow>,
    /// Detected issues.
    pub(super) issues: Vec<CrossFileReactivityIssue>,
    /// Provide definitions by component file.
    pub(super) provides: FxHashMap<FileId, Vec<ProvideDefinition>>,
}

impl<'a> CrossFileReactivityAnalyzer<'a> {
    /// Create a new analyzer.
    pub fn new(registry: &'a ModuleRegistry, graph: &'a DependencyGraph) -> Self {
        Self {
            registry,
            graph,
            flows: Vec::new(),
            issues: Vec::new(),
            provides: FxHashMap::default(),
        }
    }

    /// Run the full analysis.
    pub fn analyze(mut self) -> (Vec<CrossFileReactivityIssue>, Vec<CrossFileDiagnostic>) {
        // Phase 1: Collect provide definitions
        self.collect_provides();

        // Phase 2: Track flows across file boundaries
        self.track_cross_file_flows();

        // Phase 3: Detect issues
        self.detect_issues();

        // Generate diagnostics
        let diagnostics = self.generate_diagnostics();

        (self.issues, diagnostics)
    }
}
