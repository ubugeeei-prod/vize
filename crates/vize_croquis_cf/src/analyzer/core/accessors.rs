use super::CrossFileAnalyzer;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use std::path::Path;
use vize_croquis::Croquis;
#[cfg(test)]
use vize_croquis::EffectGraphSummary;

#[cfg(test)]
mod clear_tests;

impl CrossFileAnalyzer {
    /// Check potential original source facts without granting host authority.
    /// This performs no IO and does not suppress a diagnostic.
    #[doc(hidden)]
    pub fn has_closed_nuxt_render_facts(&self) -> bool {
        crate::rules::has_closed_nuxt_render_facts(
            &self.registry,
            &self.graph,
            self.nuxt_dirs.as_ref(),
        )
    }

    /// Get the module registry.
    #[inline]
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    /// Get the dependency graph.
    #[inline]
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Get analysis for a specific file.
    pub fn get_analysis(&self, file_id: FileId) -> Option<&Croquis> {
        self.registry.get(file_id).map(|e| &e.analysis)
    }

    /// Get file path by ID.
    pub fn get_file_path(&self, file_id: FileId) -> Option<&Path> {
        self.registry.get(file_id).map(|e| e.path.as_path())
    }

    #[cfg(test)]
    pub(crate) fn effect_graph_summary(&self, file_id: FileId) -> Option<EffectGraphSummary> {
        self.effect_graph_summaries.get(&file_id).copied()
    }

    /// Clear all data and reset.
    pub fn clear(&mut self) {
        self.nuxt_scope_once = false;
        self.registry.clear();
        self.graph = DependencyGraph::new();
        self.effect_graph_summaries.clear();
        self.template_complexity.clear();
    }

    pub(super) fn count_edges(&self) -> usize {
        self.graph.nodes().map(|node| node.imports.len()).sum()
    }
}
