use super::CrossFileAnalyzer;
use crate::analyzer::types::CrossFileOptions;
use crate::graph::DependencyGraph;
use crate::registry::ModuleRegistry;
use std::path::Path;
use vize_croquis::AnalyzerOptions;

impl CrossFileAnalyzer {
    /// Create a new cross-file analyzer.
    pub fn new(options: CrossFileOptions) -> Self {
        Self {
            options,
            ssr_enabled: true,
            registry: ModuleRegistry::new(),
            graph: DependencyGraph::new(),
            effect_graph_summaries: Default::default(),
            template_complexity: Default::default(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Create with a project root directory.
    pub fn with_project_root(options: CrossFileOptions, root: impl AsRef<Path>) -> Self {
        Self {
            options,
            ssr_enabled: true,
            registry: ModuleRegistry::with_project_root(root.as_ref()),
            graph: DependencyGraph::new(),
            effect_graph_summaries: Default::default(),
            template_complexity: Default::default(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Set the host project's SSR capability without changing enabled rule groups.
    /// Unknown projects must retain the default `true`.
    #[doc(hidden)]
    pub fn set_ssr_enabled(&mut self, enabled: bool) {
        self.ssr_enabled = enabled;
    }

    /// Set single-file analyzer options.
    pub fn set_single_file_options(&mut self, options: AnalyzerOptions) {
        self.single_file_options = options;
    }
}
