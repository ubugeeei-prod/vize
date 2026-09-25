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
    pub path_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedProviderBranch {
    Matched(ResolvedProvider),
    Unmatched {
        path: Vec<FileId>,
        path_count: usize,
    },
}

impl ResolvedProviderBranch {
    pub(crate) fn path(&self) -> &[FileId] {
        match self {
            Self::Matched(provider) => &provider.path,
            Self::Unmatched { path, .. } => path,
        }
    }

    pub(crate) fn path_count(&self) -> usize {
        match self {
            Self::Matched(provider) => provider.path_count,
            Self::Unmatched { path_count, .. } => *path_count,
        }
    }
}

pub(crate) struct ProviderResolution {
    pub branches: Vec<ResolvedProviderBranch>,
    /// Parent/child edges on a path to a nearest provider or an unmatched root.
    pub edges: Vec<(FileId, FileId)>,
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
        self.resolve_provider_resolution(consumer, key).branches
    }

    /// Collapse shared DAG ancestors while retaining exact branch counts and
    /// one deterministic representative path for each terminal outcome.
    pub(crate) fn resolve_provider_resolution(
        &self,
        consumer: FileId,
        key: &ProvideKey,
    ) -> ProviderResolution {
        let mut visited = FxHashSet::default();
        visited.insert(consumer);
        let mut queue = vec![consumer];
        let mut cursor = 0;
        let mut parents_by_node: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
        let mut predecessor = FxHashMap::default();
        let mut terminals = Vec::new();
        let mut edges = Vec::new();

        while let Some(&current) = queue.get(cursor) {
            cursor += 1;
            if current != consumer
                && self
                    .provides
                    .get(&current)
                    .and_then(|provides| matching_provider(provides, key))
                    .is_some()
            {
                terminals.push(current);
                continue;
            }

            let Some(parents) = self.component_parents.get(&current) else {
                terminals.push(current);
                continue;
            };
            if parents.is_empty() {
                terminals.push(current);
                continue;
            }
            for &parent in parents {
                edges.push((parent, current));
                predecessor.entry(parent).or_insert(current);
                if visited.insert(parent) {
                    queue.push(parent);
                }
            }
            parents_by_node.insert(current, parents.clone());
        }

        // Count all render paths through the explored DAG without materializing
        // them. A cycle needs the ancestor-sensitive behavior of the old walk.
        let mut remaining_children: FxHashMap<FileId, usize> =
            visited.iter().map(|&file_id| (file_id, 0)).collect();
        for parents in parents_by_node.values() {
            for parent in parents {
                *remaining_children.entry(*parent).or_default() += 1;
            }
        }
        let mut ready = vec![consumer];
        let mut ready_cursor = 0;
        let mut path_counts = FxHashMap::default();
        path_counts.insert(consumer, 1usize);
        while let Some(&current) = ready.get(ready_cursor) {
            ready_cursor += 1;
            let count = path_counts.get(&current).copied().unwrap_or(0);
            for &parent in parents_by_node.get(&current).into_iter().flatten() {
                let paths = path_counts.entry(parent).or_insert(0);
                *paths = paths.saturating_add(count);
                let remaining = remaining_children
                    .get_mut(&parent)
                    .expect("each explored parent has a counter");
                *remaining -= 1;
                if *remaining == 0 {
                    ready.push(parent);
                }
            }
        }
        if ready_cursor != visited.len() {
            return self.resolve_provider_paths_with_cycles(consumer, key);
        }

        let mut branches = terminals
            .into_iter()
            .map(|terminal| {
                let mut path = vec![terminal];
                let mut current = terminal;
                while current != consumer {
                    current = predecessor[&current];
                    path.push(current);
                }
                let path_count = path_counts.get(&terminal).copied().unwrap_or(1);
                if let Some(provide) = self
                    .provides
                    .get(&terminal)
                    .filter(|_| terminal != consumer)
                    .and_then(|provides| matching_provider(provides, key))
                {
                    ResolvedProviderBranch::Matched(ResolvedProvider {
                        provider_id: terminal,
                        provide: provide.clone(),
                        path,
                        path_count,
                    })
                } else {
                    ResolvedProviderBranch::Unmatched { path, path_count }
                }
            })
            .collect::<Vec<_>>();
        branches.sort_by(|left, right| self.compare_paths(left.path(), right.path()));
        ProviderResolution { branches, edges }
    }

    fn resolve_provider_paths_with_cycles(
        &self,
        consumer: FileId,
        key: &ProvideKey,
    ) -> ProviderResolution {
        let mut branches = Vec::new();
        let mut frames = vec![AncestorFrame {
            current: consumer,
            parent: None,
        }];
        let mut cursor = 0;

        while let Some(current) = frames.get(cursor).map(|frame| frame.current) {
            let frame_index = cursor;
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
                    path_count: 1,
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
                    path_count: 1,
                });
            }
        }

        branches.sort_by(|left, right| self.compare_paths(left.path(), right.path()));
        let edges = branches
            .iter()
            .flat_map(|branch| branch.path().windows(2).map(|pair| (pair[0], pair[1])))
            .collect();
        ProviderResolution { branches, edges }
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
    while let Some(&frame) = frames.get(index) {
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
        let Some(&frame) = frames.get(index) else {
            return false;
        };
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
    let provides = vize_croquis::facts::provide_entries(analysis);
    let injects = vize_croquis::facts::inject_entries(analysis);
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
