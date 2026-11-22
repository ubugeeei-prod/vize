//! The plugin host and health analyzer read the same original inline Vue template.
use vize_croquis_cf::TemplateComplexity;
use vize_l0::fact::FactManager;
use vize_patina::plugin_batch::{
    batch::{BuiltBatch, PluginSpec, build_batch},
    document::PluginDocument,
    error::HostError,
    facts::REGISTRY,
};

/// A batch and its original document for mapping plugin replies.
pub struct PluginTemplateBatch {
    pub document: PluginDocument,
    pub batch: BuiltBatch,
}

/// Independent outputs for the SDK and Doctor's component complexity report.
pub struct PluginTemplateProducts {
    pub plugin: Result<PluginTemplateBatch, HostError>,
    pub complexity: Option<TemplateComplexity>,
}

/// Run the configured plugin fact demands and Doctor's component analysis for
/// the same source. Only inline HTML templates produce complexity observations.
pub fn analyze(source: &str, filename: &str, spec: &PluginSpec<'_>) -> PluginTemplateProducts {
    let prepared = PluginDocument::with_template_pass(source, filename, |lowered| {
        let facts = vize_l1_to_l2::pass::cfg::run(lowered);
        TemplateComplexity::from_facts(&facts, lowered.artifact.source)
    });
    match prepared {
        Ok((document, complexity)) => {
            let plugin = build_batch(&document, spec, &mut FactManager::new(&REGISTRY))
                .map(|batch| PluginTemplateBatch { batch, document });
            PluginTemplateProducts { plugin, complexity }
        }
        Err(error) => PluginTemplateProducts {
            plugin: Err(error),
            complexity: None,
        },
    }
}
