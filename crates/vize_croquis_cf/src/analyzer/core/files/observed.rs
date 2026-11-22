use super::{
    Croquis, CrossFileAnalyzer, EffectGraphSummary, FileId, ModuleNode, Path,
    note_mounted_resources,
};
use crate::TemplateComplexity;

impl CrossFileAnalyzer {
    /// Observe a Vue template and register the complete module with its own
    /// complexity. The pass runs before any graph mutation and receives this
    /// revision's source. An edit with no complexity clears the old record.
    /// Other module types use the regular registration and return no observation.
    pub fn add_file_with_template_pass<T>(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: Croquis,
        effects: EffectGraphSummary,
        pass: impl FnOnce(&str) -> (Option<TemplateComplexity>, T),
    ) -> (FileId, Option<T>) {
        let path = path.as_ref();
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vue"))
        {
            let id =
                self.add_file_with_analysis_and_effect_summary(path, source, analysis, effects);
            return (id, None);
        }

        let (complexity, product) = pass(source);
        let analysis = note_mounted_resources(source, analysis);
        self.nuxt_scope_once = false;
        let (id, is_new) = self.registry.register(path, source, analysis);
        self.record_effect_graph_summary(id, effects);
        match complexity {
            Some(facts) => {
                self.template_complexity.insert(id, facts);
            }
            None => {
                self.template_complexity.remove(&id);
            }
        }
        if is_new {
            let mut node = ModuleNode::new(id, path.to_string_lossy().as_ref());
            if let Some(entry) = self.registry.get(id) {
                node.component_name = entry.component_name.clone();
            }
            node.is_entry = matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("App.vue" | "index.vue")
            );
            self.graph.add_node(node);
        }
        self.update_dependency_edges(id);
        (id, Some(product))
    }
}
