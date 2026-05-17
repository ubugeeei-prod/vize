use super::keys::create_string_key_diagnostic;
use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

#[derive(Debug)]
pub(crate) struct ProvideInjectIndex {
    provides: FxHashMap<FileId, Vec<ProvideEntry>>,
    injects: FxHashMap<FileId, Vec<InjectEntry>>,
    component_parents: FxHashMap<FileId, Vec<FileId>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedProvider {
    pub provider_id: FileId,
    pub provide: ProvideEntry,
    pub path: Vec<FileId>,
}

#[derive(Debug, Clone, Copy)]
struct AncestorFrame {
    current: FileId,
    parent: Option<usize>,
}

impl ProvideInjectIndex {
    pub(crate) fn new(registry: &ModuleRegistry, graph: &DependencyGraph) -> Self {
        let mut provides = FxHashMap::default();
        let mut injects = FxHashMap::default();

        for entry in registry.vue_components() {
            let (entry_provides, entry_injects) = extract_provide_inject(&entry.analysis);
            if !entry_provides.is_empty() {
                provides.insert(entry.id, entry_provides);
            }
            if !entry_injects.is_empty() {
                injects.insert(entry.id, entry_injects);
            }
        }

        let mut component_parents: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
        for node in graph.nodes() {
            for (child_id, edge_type) in &node.imports {
                if *edge_type == DependencyEdge::ComponentUsage {
                    component_parents
                        .entry(*child_id)
                        .or_default()
                        .push(node.file_id);
                }
            }
        }

        for parents in component_parents.values_mut() {
            parents.sort_by_key(|id| id.as_u32());
            parents.dedup();
        }

        Self {
            provides,
            injects,
            component_parents,
        }
    }

    pub(crate) fn provides(&self) -> &FxHashMap<FileId, Vec<ProvideEntry>> {
        &self.provides
    }

    pub(crate) fn injects(&self) -> &FxHashMap<FileId, Vec<InjectEntry>> {
        &self.injects
    }

    pub(crate) fn string_key_diagnostics(&self) -> Vec<CrossFileDiagnostic> {
        let mut diagnostics = Vec::new();

        for (&file_id, provides) in &self.provides {
            for provide in provides {
                if let ProvideKey::String(key) = &provide.key {
                    diagnostics.push(create_string_key_diagnostic(
                        file_id,
                        key,
                        true,
                        provide.start,
                        provide.end,
                    ));
                }
            }
        }

        for (&file_id, injects) in &self.injects {
            for inject in injects {
                if let ProvideKey::String(key) = &inject.key {
                    diagnostics.push(create_string_key_diagnostic(
                        file_id,
                        key,
                        false,
                        inject.start,
                        inject.end,
                    ));
                }
            }
        }

        diagnostics
    }

    /// Find the nearest providers for a given key in every ancestor branch.
    pub(crate) fn resolve_providers(
        &self,
        consumer: FileId,
        key: &ProvideKey,
    ) -> Vec<ResolvedProvider> {
        let mut matches = Vec::new();
        let mut seen_providers = FxHashSet::default();
        let mut frames = vec![AncestorFrame {
            current: consumer,
            parent: None,
        }];
        let mut cursor = 0;

        while cursor < frames.len() {
            let frame_index = cursor;
            let current = frames[frame_index].current;
            cursor += 1;

            // A provider shadows farther ancestors on the same render branch.
            if current != consumer
                && let Some(component_provides) = self.provides.get(&current)
                && let Some(provide) = matching_provider(component_provides, key)
            {
                if seen_providers.insert((current, provide.id.as_u32())) {
                    matches.push(ResolvedProvider {
                        provider_id: current,
                        provide: provide.clone(),
                        path: path_from_frame(&frames, frame_index),
                    });
                }
                continue;
            }

            let Some(parents) = self.component_parents.get(&current) else {
                continue;
            };

            for &parent_id in parents {
                if frame_contains(&frames, frame_index, parent_id) {
                    continue;
                }
                frames.push(AncestorFrame {
                    current: parent_id,
                    parent: Some(frame_index),
                });
            }
        }

        matches.sort_by_key(|provider| {
            (
                provider.path.len(),
                provider.provider_id.as_u32(),
                provider.provide.id.as_u32(),
            )
        });
        matches
    }
}

fn matching_provider<'a>(
    component_provides: &'a [ProvideEntry],
    key: &ProvideKey,
) -> Option<&'a ProvideEntry> {
    component_provides
        .iter()
        .rev()
        .find(|provide| provide.key == *key)
}

fn path_from_frame(frames: &[AncestorFrame], mut index: usize) -> Vec<FileId> {
    let mut path = Vec::new();
    loop {
        let frame = frames[index];
        path.push(frame.current);
        let Some(parent) = frame.parent else {
            break;
        };
        index = parent;
    }
    path
}

fn frame_contains(frames: &[AncestorFrame], mut index: usize, needle: FileId) -> bool {
    loop {
        let frame = frames[index];
        if frame.current == needle {
            return true;
        }
        let Some(parent) = frame.parent else {
            return false;
        };
        index = parent;
    }
}

/// Extract provide/inject calls from a component's analysis.
/// Uses the ProvideInjectTracker for precise static analysis - no heuristics.
#[inline]
fn extract_provide_inject(
    analysis: &vize_croquis::Croquis,
) -> (Vec<ProvideEntry>, Vec<InjectEntry>) {
    // Use the actual provide/inject tracker data - precise static analysis
    let provides = analysis.provide_inject.provides().to_vec();
    let injects = analysis.provide_inject.injects().to_vec();
    (provides, injects)
}
