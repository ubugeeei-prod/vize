//! Core state for cross-file reactivity tracking.

use super::types::{
    ComposableInfo, CrossFileReactiveValue, CrossFileReactivityIssue, ProvideDefinition,
    ReactiveValueId, ReactivityFlow,
};
use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::FxHashMap;

pub struct CrossFileReactivityAnalyzer<'a> {
    pub(super) registry: &'a ModuleRegistry,
    pub(super) graph: &'a DependencyGraph,
    /// All tracked reactive values.
    pub(super) reactive_values: FxHashMap<ReactiveValueId, CrossFileReactiveValue>,
    /// Reactivity flows between files.
    pub(super) flows: Vec<ReactivityFlow>,
    /// Detected issues.
    pub(super) issues: Vec<CrossFileReactivityIssue>,
    /// Composable definitions (file -> composable name -> return type info).
    pub(super) composables: FxHashMap<FileId, Vec<ComposableInfo>>,
    /// Provide definitions by component file.
    pub(super) provides: FxHashMap<FileId, Vec<ProvideDefinition>>,
}

impl<'a> CrossFileReactivityAnalyzer<'a> {
    /// Create a new analyzer.
    pub fn new(registry: &'a ModuleRegistry, graph: &'a DependencyGraph) -> Self {
        Self {
            registry,
            graph,
            reactive_values: FxHashMap::default(),
            flows: Vec::new(),
            issues: Vec::new(),
            composables: FxHashMap::default(),
            provides: FxHashMap::default(),
        }
    }

    /// Run the full analysis.
    pub fn analyze(mut self) -> (Vec<CrossFileReactivityIssue>, Vec<CrossFileDiagnostic>) {
        // Phase 1: Collect all reactive value definitions
        self.collect_reactive_definitions();

        // Phase 2: Collect composable definitions
        self.collect_composables();

        // Phase 3: Collect provide definitions
        self.collect_provides();

        // Phase 4: Track flows across file boundaries
        self.track_cross_file_flows();

        // Phase 5: Detect issues
        self.detect_issues();

        // Generate diagnostics
        let diagnostics = self.generate_diagnostics();

        (self.issues, diagnostics)
    }
}
