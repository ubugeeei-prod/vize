mod slot_tree;

use crate::facts::imported_render_target;
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use std::borrow::Cow;
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::provide::{ProvideEntry, ProvideKey};

#[derive(Debug, Default)]
pub(super) struct SlotScopes {
    scopes: FxHashMap<FileId, Vec<SlotScope>>,
    receivers: FxHashSet<FileId>,
    bare_parents: FxHashMap<FileId, FxHashSet<FileId>>,
}

#[derive(Debug)]
struct SlotScope {
    owner: FileId,
    receivers: Vec<FileId>,
}

impl SlotScopes {
    fn record(&mut self, owner: FileId, usages: &[RuntimeUsage]) {
        if !usages.iter().any(|usage| usage.renders_slot) {
            return;
        }
        let hosts: Vec<_> = (0..usages.len())
            .map(|index| nearest_containing_usage(usages, index))
            .collect();
        let projected: FxHashSet<_> = usages
            .iter()
            .zip(&hosts)
            .filter(|(_, host)| {
                host.and_then(|index| usages.get(index))
                    .is_some_and(|usage| usage.renders_slot)
            })
            .map(|(usage, _)| usage.target_id)
            .collect();
        for (index, usage) in usages.iter().enumerate() {
            // Record every use of a projected component, including a bare use
            // in the same file, so slot matching cannot hide a real root branch.
            if !projected.contains(&usage.target_id) {
                continue;
            }
            let mut receivers = Vec::new();
            let mut current = index;
            let mut mounted = true;
            while let Some(host_index) = hosts.get(current).copied().flatten() {
                let Some(host) = usages.get(host_index) else {
                    break;
                };
                if !host.renders_slot {
                    // Preserve the existing lexical fallback past a hidden
                    // outer wrapper, without adding a receiver from that use.
                    mounted = current != index;
                    receivers.clear();
                    break;
                }
                receivers.push(host.target_id);
                current = host_index;
            }
            if mounted {
                self.receivers.extend(receivers.iter().copied());
                self.scopes
                    .entry(usage.target_id)
                    .or_default()
                    .push(SlotScope { owner, receivers });
            }
        }
    }

    pub(super) fn parents<'a>(
        &self,
        child: FileId,
        key: &ProvideKey,
        provides: &FxHashMap<FileId, Vec<ProvideEntry>>,
        fallback: &'a [FileId],
        order: &FxHashMap<FileId, usize>,
    ) -> Cow<'a, [FileId]> {
        let Some(scopes) = self.scopes.get(&child) else {
            return Cow::Borrowed(fallback);
        };
        let receiver = |scope: &SlotScope| {
            scope.receivers.iter().copied().find(|&host| {
                host != child
                    && provides
                        .get(&host)
                        .and_then(|entries| super::matching_provider(entries, key))
                        .is_some()
            })
        };
        // Preserve the existing owner paths, counts, and allocations when no
        // receiver provides this key. Other uses of a wrapper are not slot uses.
        if !scopes.iter().any(|scope| receiver(scope).is_some()) {
            return Cow::Borrowed(fallback);
        }
        let mut parents: Vec<_> = fallback
            .iter()
            .copied()
            .filter(|parent| !scopes.iter().any(|scope| scope.owner == *parent))
            .collect();
        parents.extend(
            scopes
                .iter()
                .map(|scope| receiver(scope).unwrap_or(scope.owner)),
        );
        parents.sort_by_key(|id| (stable_rank(order, *id), id.as_u32()));
        parents.dedup();
        Cow::Owned(parents)
    }

    pub(super) fn is_receiver(&self, file: FileId) -> bool {
        self.receivers.contains(&file)
    }

    pub(super) fn tree_edges<'a>(
        &self,
        edges: &'a [(FileId, FileId)],
        provides: &FxHashMap<FileId, Vec<ProvideEntry>>,
    ) -> Cow<'a, [(FileId, FileId)]> {
        if !self.scopes.values().flatten().any(|scope| {
            scope
                .receivers
                .iter()
                .any(|host| provides.contains_key(host))
        }) {
            return Cow::Borrowed(edges);
        }
        let mut resolved = Vec::with_capacity(edges.len());
        for &(parent, child) in edges {
            let mut projected = false;
            for scope in self.scopes.get(&child).into_iter().flatten() {
                let end = if scope.owner == parent {
                    Some(scope.receivers.len())
                } else {
                    scope.receivers.iter().position(|host| *host == parent)
                };
                let Some(end) = end else { continue };
                projected = true;
                // Keep the writer in the displayed ancestry, so reusing a
                // receiver without this slot cannot acquire its slot children.
                let mut owner = scope.owner;
                for &host in scope.receivers.iter().rev() {
                    if host != owner && provides.contains_key(&host) {
                        resolved.push((owner, host));
                        owner = host;
                    }
                }
                let mut current = parent;
                for &host in scope.receivers.iter().take(end).rev() {
                    if host != current && provides.contains_key(&host) {
                        resolved.push((current, host));
                        current = host;
                    }
                }
                if current != child {
                    resolved.push((current, child));
                }
            }
            if !projected {
                resolved.push((parent, child));
            }
        }
        Cow::Owned(resolved)
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimeUsage {
    target_id: FileId,
    start: u32,
    renders_slot: bool,
}

pub(super) fn runtime_component_parents(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    stable_file_order: &FxHashMap<FileId, usize>,
) -> (FxHashMap<FileId, Vec<FileId>>, SlotScopes) {
    let mut component_parents: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
    let mut slot_scopes = SlotScopes::default();

    for entry in registry.vue_components() {
        if vize_croquis::facts::component_usage_list(&entry.analysis).is_empty() {
            add_graph_component_parents(&mut component_parents, graph, entry.id);
            continue;
        }

        let usages = runtime_usages(entry, registry, graph);
        if usages.is_empty() {
            add_graph_component_parents(&mut component_parents, graph, entry.id);
            continue;
        }
        slot_scopes.record(entry.id, &usages);

        for (index, usage) in usages.iter().enumerate() {
            match nearest_containing_usage(&usages, index).and_then(|host| usages.get(host)) {
                // Content nested in a component with no `<slot>` is not mounted.
                // Content in a `<slot>` is created by this file, not by the
                // wrapper, so other uses of the wrapper are not extra branches.
                Some(host) if !host.renders_slot => {}
                host => {
                    add_component_parent(&mut component_parents, usage.target_id, entry.id);
                    if host.is_none() {
                        slot_scopes
                            .bare_parents
                            .entry(usage.target_id)
                            .or_default()
                            .insert(entry.id);
                    }
                }
            }
        }
    }

    for parents in component_parents.values_mut() {
        parents.sort_by_key(|id| (stable_rank(stable_file_order, *id), id.as_u32()));
        parents.dedup();
    }

    (component_parents, slot_scopes)
}

pub(super) fn stable_file_order(registry: &ModuleRegistry) -> FxHashMap<FileId, usize> {
    let mut entries = registry
        .vue_components()
        .map(|entry| (entry.path.clone(), entry.id))
        .collect::<Vec<_>>();
    entries.sort_by(|(left_path, left_id), (right_path, right_id)| {
        left_path
            .cmp(right_path)
            .then_with(|| left_id.as_u32().cmp(&right_id.as_u32()))
    });
    entries
        .into_iter()
        .enumerate()
        .map(|(rank, (_, file_id))| (file_id, rank))
        .collect()
}

pub(super) fn stable_rank(stable_file_order: &FxHashMap<FileId, usize>, file_id: FileId) -> usize {
    stable_file_order
        .get(&file_id)
        .copied()
        .unwrap_or(usize::MAX)
}

fn runtime_usages(
    entry: &ModuleEntry,
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> Vec<RuntimeUsage> {
    vize_croquis::facts::component_usage_list(&entry.analysis)
        .iter()
        .filter_map(|usage| {
            let target_id = imported_render_target(registry, entry.id, usage.name.as_str())
                .or_else(|| graph.find_by_component(usage.name.as_str()))?;
            Some(RuntimeUsage {
                target_id,
                start: usage.start,
                renders_slot: registry.renders_slot(target_id),
            })
        })
        .collect()
}

fn nearest_containing_usage(usages: &[RuntimeUsage], child_index: usize) -> Option<usize> {
    let child_start = usages.get(child_index)?.start;
    usages
        .iter()
        .enumerate()
        .filter(|(index, usage)| {
            // Component usages are collected in postorder: ancestors appear
            // after their descendants and start earlier in the template.
            *index > child_index && usage.start < child_start
        })
        .max_by_key(|(_, usage)| usage.start)
        .map(|(index, _)| index)
}

fn add_graph_component_parents(
    component_parents: &mut FxHashMap<FileId, Vec<FileId>>,
    graph: &DependencyGraph,
    parent_id: FileId,
) {
    let Some(node) = graph.get_node(parent_id) else {
        return;
    };

    for (child_id, edge_type) in &node.imports {
        if *edge_type == DependencyEdge::ComponentUsage {
            add_component_parent(component_parents, *child_id, parent_id);
        }
    }
}

fn add_component_parent(
    component_parents: &mut FxHashMap<FileId, Vec<FileId>>,
    child_id: FileId,
    parent_id: FileId,
) {
    if child_id != parent_id {
        component_parents
            .entry(child_id)
            .or_default()
            .push(parent_id);
    }
}
