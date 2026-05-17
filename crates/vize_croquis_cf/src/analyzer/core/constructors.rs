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
            registry: ModuleRegistry::new(),
            graph: DependencyGraph::new(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Create with a project root directory.
    pub fn with_project_root(options: CrossFileOptions, root: impl AsRef<Path>) -> Self {
        Self {
            options,
            registry: ModuleRegistry::with_project_root(root.as_ref()),
            graph: DependencyGraph::new(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Set single-file analyzer options.
    pub fn set_single_file_options(&mut self, options: AnalyzerOptions) {
        self.single_file_options = options;
    }
}
