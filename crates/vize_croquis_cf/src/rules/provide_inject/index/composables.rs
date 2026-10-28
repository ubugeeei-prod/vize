use super::calls::extract_provide_inject;
use crate::facts::resolve_module;
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use crate::rules::cross_file_reactivity::provided_value_reactive_kind;
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, InjectPattern, ProvideEntry, ProviderId};

pub(super) struct ComposableProjection<'a> {
    registry: &'a ModuleRegistry,
    effects: FxHashMap<(FileId, CompactString), (Vec<ProvideEntry>, Vec<InjectEntry>)>,
}

impl<'a> ComposableProjection<'a> {
    pub(super) fn new(registry: &'a ModuleRegistry) -> Self {
        Self {
            registry,
            effects: FxHashMap::default(),
        }
    }

    pub(super) fn project(
        &mut self,
        entry: &ModuleEntry,
        entry_provides: &mut Vec<ProvideEntry>,
        entry_injects: &mut Vec<InjectEntry>,
        reactive_provides: &mut FxHashSet<(FileId, u32)>,
    ) {
        let mut next_provider_id = entry_provides
            .iter()
            .map(|provide| provide.id.as_u32())
            .max()
            .map_or(0, |id| id.saturating_add(1));

        for call in vize_croquis::facts::composable_calls(&entry.analysis) {
            if !is_local_module_specifier(call.source.as_str()) {
                continue;
            }
            let Some(source_id) = resolve_module(
                self.registry,
                call.source.as_str(),
                entry.path.parent(),
            ) else {
                continue;
            };
            let Some(source) = self.registry.get(source_id) else {
                continue;
            };
            let (source_provides, source_injects) = self
                .effects
                .entry((source_id, call.name.clone()))
                .or_insert_with(|| {
                    let (mut provides, mut injects) = extract_provide_inject(&source.analysis);
                    provides.retain(|provide| {
                        provide.from_composable.as_deref() == Some(call.name.as_str())
                    });
                    injects.retain(|inject| {
                        inject.from_composable.as_deref() == Some(call.name.as_str())
                    });
                    (provides, injects)
                });

            for mut provide in source_provides.iter().cloned() {
                provide.id = ProviderId::new(next_provider_id);
                next_provider_id = next_provider_id.saturating_add(1);
                provide.start = call.start;
                provide.end = call.end;
                if provided_value_reactive_kind(&source.analysis, provide.value.as_str()).is_some() {
                    reactive_provides.insert((entry.id, provide.id.as_u32()));
                }
                entry_provides.push(provide);
            }

            for mut inject in source_injects.iter().cloned() {
                inject.start = call.start;
                inject.end = call.end;
                if let InjectPattern::IndirectDestructure { offset, .. } = &mut inject.pattern {
                    *offset = call.start;
                }
                entry_injects.push(inject);
            }
        }
    }
}

fn is_local_module_specifier(specifier: &str) -> bool {
    specifier.starts_with('.') || specifier.starts_with("@/") || specifier.starts_with('/')
}
