use super::engine::CrossFileReactivityAnalyzer;
use super::provide_helpers::{classify_provided_value, provide_key_display, provide_key_identity};
use super::types::ProvideDefinition;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    /// Collect provide() definitions.
    pub(super) fn collect_provides(&mut self) {
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            for provide in vize_croquis::facts::provide_entries(analysis) {
                let key_str = provide_key_display(&provide.key);
                let key_identity = provide_key_identity(&provide.key);

                // Check if the provided value is reactive
                let classification = classify_provided_value(analysis, provide.value.as_str());

                self.provides
                    .entry(file_id)
                    .or_default()
                    .push(ProvideDefinition {
                        file_id,
                        key: key_str,
                        key_identity,
                        value_name: provide.value.clone(),
                        is_reactive: classification.reactive_kind.is_some(),
                        is_reactive_service: classification.is_reactive_service,
                        offset: provide.start,
                    });
            }
        }
    }
}
