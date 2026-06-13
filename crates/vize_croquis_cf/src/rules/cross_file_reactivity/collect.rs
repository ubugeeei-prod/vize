use super::engine::CrossFileReactivityAnalyzer;
use super::provide_helpers::{
    provide_key_display, provide_key_identity, provided_value_reactive_kind,
};
use super::types::{ComposableInfo, CrossFileReactiveValue, ProvideDefinition, ReactiveValueId};
use vize_carton::{CompactString, SmallVec};
use vize_croquis::reactivity::ReactiveKind;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn collect_reactive_definitions(&mut self) {
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            // Collect from reactivity sources
            for source in analysis.reactivity.sources() {
                let id = ReactiveValueId {
                    file_id,
                    name: source.name.clone(),
                    offset: source.declaration_offset,
                };

                self.reactive_values.insert(
                    id.clone(),
                    CrossFileReactiveValue {
                        id,
                        kind: source.kind,
                        exposures: SmallVec::new(),
                        consumptions: SmallVec::new(),
                        reactivity_preserved: true,
                    },
                );
            }
        }

        // Also collect from TypeScript/JavaScript modules
        for entry in self.registry.iter().filter(|e| !e.is_vue_sfc) {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            for source in analysis.reactivity.sources() {
                let id = ReactiveValueId {
                    file_id,
                    name: source.name.clone(),
                    offset: source.declaration_offset,
                };

                self.reactive_values.insert(
                    id.clone(),
                    CrossFileReactiveValue {
                        id,
                        kind: source.kind,
                        exposures: SmallVec::new(),
                        consumptions: SmallVec::new(),
                        reactivity_preserved: true,
                    },
                );
            }
        }
    }

    /// Phase 2: Collect composable function definitions.
    pub(super) fn collect_composables(&mut self) {
        // Composables are typically in .ts files with "use" prefix
        for entry in self.registry.iter().filter(|e| !e.is_vue_sfc) {
            let file_id = entry.id;
            let path = entry.path.to_string_lossy();
            let path_str = path.as_ref();

            // Check if this looks like a composable file
            let filename = path_str.rsplit('/').next().unwrap_or(path_str);
            if !filename.starts_with("use") && !path_str.contains("/composables/") {
                continue;
            }

            let analysis = &entry.analysis;
            let mut composable_infos = Vec::new();

            // Look for exported functions that start with "use"
            for scope in analysis.scopes.iter() {
                if let vize_croquis::ScopeKind::Function = scope.kind {
                    for (name, _) in scope.bindings() {
                        if name.starts_with("use") {
                            // This is likely a composable
                            // Collect its reactive returns
                            let reactive_returns: Vec<(CompactString, ReactiveKind)> = analysis
                                .reactivity
                                .sources()
                                .iter()
                                .map(|s| (s.name.clone(), s.kind))
                                .collect();

                            composable_infos.push(ComposableInfo {
                                name: CompactString::new(name),
                                reactive_returns,
                                file_id,
                                offset: scope.span.start,
                            });
                        }
                    }
                }
            }

            if !composable_infos.is_empty() {
                self.composables.insert(file_id, composable_infos);
            }
        }
    }

    /// Phase 3: Collect provide() definitions.
    pub(super) fn collect_provides(&mut self) {
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            for provide in analysis.provide_inject.provides() {
                let key_str = provide_key_display(&provide.key);
                let key_identity = provide_key_identity(&provide.key);

                // Check if the provided value is reactive
                let reactive_kind = provided_value_reactive_kind(analysis, provide.value.as_str());
                let is_reactive = reactive_kind.is_some();

                self.provides
                    .entry(file_id)
                    .or_default()
                    .push(ProvideDefinition {
                        file_id,
                        key: key_str,
                        key_identity,
                        value_name: provide.value.clone(),
                        is_reactive,
                        reactive_kind,
                        offset: provide.start,
                    });
            }
        }
    }
}
