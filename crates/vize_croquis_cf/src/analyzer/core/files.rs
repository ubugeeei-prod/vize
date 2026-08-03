use super::CrossFileAnalyzer;
use crate::graph::ModuleNode;
use crate::registry::FileId;
use std::path::Path;
use vize_croquis::{
    Croquis, EffectGraphScript, EffectGraphSummary, build_effect_graph_from_sfc_scripts,
};

impl CrossFileAnalyzer {
    /// Add a raw JavaScript, JSX, TypeScript, or TSX module to be analyzed.
    ///
    /// SFC containers must be parsed by the caller; use
    /// [`Self::add_file_with_analysis_and_effect_summary`] for `.vue` source.
    pub fn add_file(&mut self, path: impl AsRef<Path>, source: &str) -> FileId {
        let path = path.as_ref();

        // Analyze the file with single-file analyzer
        let analysis = self.analyze_single_file(source, path);
        self.add_file_with_analysis(path, source, analysis)
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
    /// `source` must be the corresponding raw script module when automatic effect
    /// summary construction is desired. SFC callers should use the explicit-summary API.
    pub fn add_file_with_analysis(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: Croquis,
    ) -> FileId {
        let effect_summary = automatic_effect_summary(path.as_ref(), source);
        self.add_file_with_analysis_and_effect_summary(path, source, analysis, effect_summary)
    }

    /// Add a file with pre-computed semantic analysis and effect summary.
    ///
    /// SFC callers should prefer this API after parsing the descriptor and
    /// summarizing the actual `<script>` and `<script setup>` block contents.
    /// This avoids treating the full SFC container as JavaScript or TypeScript.
    pub fn add_file_with_analysis_and_effect_summary(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: Croquis,
        effect_summary: EffectGraphSummary,
    ) -> FileId {
        let path = path.as_ref();

        // Register in module registry (takes ownership of analysis)
        let (file_id, is_new) = self.registry.register(path, source, analysis);
        self.record_effect_graph_summary(file_id, effect_summary);

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

    fn record_effect_graph_summary(&mut self, file_id: FileId, effect_summary: EffectGraphSummary) {
        self.effect_graph_summaries.insert(file_id, effect_summary);
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
    /// Re-resolve imports after all files have been added so each component
    /// usage follows the importing file's own binding before name fallback.
    pub fn rebuild_component_edges(&mut self) {
        self.rebuild_import_edges();
    }
}

fn automatic_effect_summary(path: &Path, source: &str) -> EffectGraphSummary {
    let extension = path.extension().and_then(|extension| extension.to_str());
    let lang = match extension {
        Some("js" | "jsx" | "ts" | "tsx") => extension,
        _ => Some("ts"),
    };
    build_effect_graph_from_sfc_scripts(None, Some(EffectGraphScript::new(source, lang))).summary()
}
