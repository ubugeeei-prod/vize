use super::CrossFileAnalyzer;
use crate::graph::{DependencyEdge, ModuleNode};
use crate::registry::FileId;
use std::path::Path;
use vize_croquis::Croquis;

impl CrossFileAnalyzer {
    /// Add a file to be analyzed.
    pub fn add_file(&mut self, path: impl AsRef<Path>, source: &str) -> FileId {
        let path = path.as_ref();

        // Analyze the file with single-file analyzer
        let analysis = self.analyze_single_file(source, path);

        // Register in module registry (takes ownership of analysis)
        let (file_id, is_new) = self.registry.register(path, source, analysis);

        if is_new {
            // Add to dependency graph
            let mut node = ModuleNode::new(file_id, path.to_string_lossy().as_ref());

            // Extract component name
            if let Some(entry) = self.registry.get(file_id) {
                node.component_name = entry.component_name.clone();
            }

            // Mark entry points
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename == "App.vue"
                || filename == "main.ts"
                || filename == "main.js"
                || filename == "index.vue"
            {
                node.is_entry = true;
            }

            self.graph.add_node(node);
        }

        self.update_dependency_edges(file_id);

        file_id
    }

    /// Add multiple files.
    pub fn add_files(&mut self, files: &[(&Path, &str)]) {
        for (path, source) in files {
            self.add_file(path, source);
        }
    }

    /// Add a file with pre-computed analysis.
    ///
    /// This is useful when the caller has already performed analysis (e.g., WASM bindings
    /// that parse both script and template content). The analysis should include
    /// `used_components` populated from template analysis for component usage edges.
    pub fn add_file_with_analysis(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: Croquis,
    ) -> FileId {
        let path = path.as_ref();

        // Register in module registry (takes ownership of analysis)
        let (file_id, is_new) = self.registry.register(path, source, analysis);

        if is_new {
            // Add to dependency graph
            let mut node = ModuleNode::new(file_id, path.to_string_lossy().as_ref());

            // Extract component name
            if let Some(entry) = self.registry.get(file_id) {
                node.component_name = entry.component_name.clone();
            }

            // Mark entry points
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename == "App.vue"
                || filename == "main.ts"
                || filename == "main.js"
                || filename == "index.vue"
            {
                node.is_entry = true;
            }

            self.graph.add_node(node);
        }

        self.update_dependency_edges(file_id);

        file_id
    }

    /// Rebuild import and import-backed component usage edges.
    ///
    /// This should be called after all files have been registered when callers
    /// add files in an arbitrary order. A parent may be added before the file
    /// referenced by `./Child.vue`; the first pass cannot resolve that target.
    pub fn rebuild_import_edges(&mut self) {
        let file_ids: Vec<_> = self.registry.iter().map(|entry| entry.id).collect();
        for file_id in file_ids {
            self.update_dependency_edges(file_id);
        }
    }

    /// Rebuild component usage edges.
    ///
    /// This should be called after all files have been added to ensure
    /// that ComponentUsage edges are correctly established. When files
    /// are added one by one, component references might not resolve
    /// if the target component hasn't been added yet.
    pub fn rebuild_component_edges(&mut self) {
        // Collect all used_components from all files
        let component_data: Vec<_> = self
            .registry
            .iter()
            .map(|entry| {
                let components: Vec<_> = entry.analysis.used_components.iter().cloned().collect();
                (entry.id, components)
            })
            .collect();

        // Add ComponentUsage edges for any that were missed
        for (file_id, used_components) in component_data {
            for component in used_components {
                if let Some(target_id) = self.find_component_by_name(component.as_str()) {
                    // add_edge checks for duplicates internally
                    self.graph
                        .add_edge(file_id, target_id, DependencyEdge::ComponentUsage);
                }
            }
        }
    }
}
