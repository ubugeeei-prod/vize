use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use vize_carton::FxHashMap;

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
) -> FxHashMap<FileId, Vec<FileId>> {
    let mut component_parents: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();

    for entry in registry.vue_components() {
        if entry.analysis.component_usages.is_empty() {
            add_graph_component_parents(&mut component_parents, graph, entry.id);
            continue;
        }

        let usages = runtime_usages(entry, registry, graph);
        if usages.is_empty() {
            add_graph_component_parents(&mut component_parents, graph, entry.id);
            continue;
        }

        for (index, usage) in usages.iter().enumerate() {
            match nearest_containing_usage(&usages, index) {
                Some(host) if host.renders_slot => {
                    add_component_parent(&mut component_parents, usage.target_id, host.target_id);
                }
                Some(_) => {}
                None => add_component_parent(&mut component_parents, usage.target_id, entry.id),
            }
        }
    }

    for parents in component_parents.values_mut() {
        parents.sort_by_key(|id| (stable_rank(stable_file_order, *id), id.as_u32()));
        parents.dedup();
    }

    component_parents
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
    entry
        .analysis
        .component_usages
        .iter()
        .filter_map(|usage| {
            let target_id = graph.find_by_component(usage.name.as_str())?;
            Some(RuntimeUsage {
                target_id,
                start: usage.start,
                renders_slot: registry.renders_slot(target_id),
            })
        })
        .collect()
}

fn nearest_containing_usage(usages: &[RuntimeUsage], child_index: usize) -> Option<&RuntimeUsage> {
    let child_start = usages[child_index].start;
    usages
        .iter()
        .enumerate()
        .filter(|(index, usage)| {
            // Component usages are collected in postorder: ancestors appear
            // after their descendants and start earlier in the template.
            *index > child_index && usage.start < child_start
        })
        .max_by_key(|(_, usage)| usage.start)
        .map(|(_, usage)| usage)
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
