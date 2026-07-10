use super::keys::create_string_key_diagnostic;
use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use std::cmp::Ordering;
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

mod parents;
use parents::{runtime_component_parents, stable_file_order, stable_rank};

#[derive(Debug)]
pub(crate) struct ProvideInjectIndex {
    provides: FxHashMap<FileId, Vec<ProvideEntry>>,
    injects: FxHashMap<FileId, Vec<InjectEntry>>,
    component_parents: FxHashMap<FileId, Vec<FileId>>,
    stable_file_order: FxHashMap<FileId, usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedProvider {
    pub provider_id: FileId,
    pub provide: ProvideEntry,
    pub path: Vec<FileId>,
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedProviderBranch {
    Matched(ResolvedProvider),
    Unmatched { path: Vec<FileId> },
}

impl ResolvedProviderBranch {
    pub(crate) fn path(&self) -> &[FileId] {
        match self {
            Self::Matched(provider) => &provider.path,
            Self::Unmatched { path } => path,
        }
    }
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

        let stable_file_order = stable_file_order(registry);
        let component_parents = runtime_component_parents(registry, graph, &stable_file_order);

        Self {
            provides,
            injects,
            component_parents,
            stable_file_order,
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

    /// Find one representative match per provider call.
    ///
    /// Branch-aware consumers should use [`Self::resolve_provider_branches`].
    pub(crate) fn resolve_providers(
        &self,
        consumer: FileId,
        key: &ProvideKey,
    ) -> Vec<ResolvedProvider> {
        let mut seen_providers = FxHashSet::default();
        self.resolve_provider_branches(consumer, key)
            .into_iter()
            .filter_map(|branch| match branch {
                ResolvedProviderBranch::Matched(provider)
                    if seen_providers
                        .insert((provider.provider_id, provider.provide.id.as_u32())) =>
                {
                    Some(provider)
                }
                _ => None,
            })
            .collect()
    }

    /// Resolve the nearest provider, or lack of one, for every ancestor branch.
    pub(crate) fn resolve_provider_branches(
        &self,
        consumer: FileId,
        key: &ProvideKey,
    ) -> Vec<ResolvedProviderBranch> {
        let mut branches = Vec::new();
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
                branches.push(ResolvedProviderBranch::Matched(ResolvedProvider {
                    provider_id: current,
                    provide: provide.clone(),
                    path: path_from_frame(&frames, frame_index),
                }));
                continue;
            }

            let mut explored_parent = false;
            for &parent_id in self.component_parents.get(&current).into_iter().flatten() {
                if frame_contains(&frames, frame_index, parent_id) {
                    continue;
                }
                explored_parent = true;
                frames.push(AncestorFrame {
                    current: parent_id,
                    parent: Some(frame_index),
                });
            }

            if !explored_parent {
                branches.push(ResolvedProviderBranch::Unmatched {
                    path: path_from_frame(&frames, frame_index),
                });
            }
        }

        branches.sort_by(|left, right| self.compare_paths(left.path(), right.path()));
        branches
    }

    pub(crate) fn sort_file_ids(&self, file_ids: &mut [FileId]) {
        file_ids.sort_by(|left, right| self.compare_files(*left, *right));
    }

    fn compare_paths(&self, left: &[FileId], right: &[FileId]) -> Ordering {
        left.len().cmp(&right.len()).then_with(|| {
            left.iter()
                .zip(right)
                .map(|(left, right)| self.compare_files(*left, *right))
                .find(|ordering| !ordering.is_eq())
                .unwrap_or(Ordering::Equal)
        })
    }

    pub(crate) fn compare_files(&self, left: FileId, right: FileId) -> Ordering {
        stable_rank(&self.stable_file_order, left)
            .cmp(&stable_rank(&self.stable_file_order, right))
            .then_with(|| left.as_u32().cmp(&right.as_u32()))
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

#[cfg(test)]
mod tests {
    use super::ProvideInjectIndex;
    use crate::registry::FileId;
    use std::cmp::Ordering;
    use vize_carton::FxHashMap;

    #[test]
    fn shared_file_order_places_known_paths_before_missing_entries() {
        let first = FileId::new(7);
        let second = FileId::new(3);
        let missing_low = FileId::new(1);
        let missing_high = FileId::new(9);
        let index = ProvideInjectIndex {
            provides: FxHashMap::default(),
            injects: FxHashMap::default(),
            component_parents: FxHashMap::default(),
            stable_file_order: FxHashMap::from_iter([(first, 0), (second, 1)]),
        };

        assert_eq!(index.compare_files(first, second), Ordering::Less);
        assert_eq!(index.compare_files(second, missing_low), Ordering::Less);
        assert_eq!(
            index.compare_files(missing_low, missing_high),
            Ordering::Less
        );
    }
}
