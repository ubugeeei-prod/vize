use super::index::ProvideInjectIndex;
use super::keys::provide_key_identity;
use super::types::{InjectInfo, ProvideInfo, ProvideInjectBranch, ProvideInjectTree, ProvideNode};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

pub(crate) fn build_provide_inject_tree_with_index(
    registry: &ModuleRegistry,
    index: &ProvideInjectIndex,
    branches: &[ProvideInjectBranch],
    edges: &[(FileId, FileId)],
) -> ProvideInjectTree {
    let mut consumer_counts: FxHashMap<(FileId, u32), usize> = FxHashMap::default();

    for branch in branches {
        if let (Some(provider), Some(provide_offset)) = (branch.provider, branch.provide_offset) {
            let count = consumer_counts
                .entry((provider, provide_offset))
                .or_insert(0);
            *count = count.saturating_add(branch.path_count);
        }
    }

    // Build the displayed tree from both matched and terminal unmatched paths.
    // This keeps pass-through components visible even when they do not provide
    // or inject the key themselves.
    let mut included_nodes = FxHashSet::default();
    let mut child_map: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
    let mut nodes_with_parent = FxHashSet::default();

    for &(parent, child) in edges {
        included_nodes.insert(parent);
        included_nodes.insert(child);
        child_map.entry(parent).or_default().push(child);
        nodes_with_parent.insert(child);
    }
    for branch in branches {
        included_nodes.extend(branch.path.iter().copied());
    }

    for &file_id in index.provides().keys() {
        included_nodes.insert(file_id);
    }
    for &file_id in index.injects().keys() {
        included_nodes.insert(file_id);
    }

    for children in child_map.values_mut() {
        children.sort_by(|left, right| index.compare_files(*left, *right));
        children.dedup();
    }

    let root_ids = select_root_ids(
        index,
        &included_nodes,
        &nodes_with_parent,
        &child_map,
        branches,
    );

    let roots = root_ids
        .into_iter()
        .map(|file_id| {
            let mut active_nodes = FxHashSet::default();
            let mut expanded = FxHashSet::default();
            // Cycles have no natural root. A synthetic root only supplies
            // context for its descendants; its own injects belong to the
            // occurrence reached through another component.
            build_node(
                file_id,
                registry,
                &child_map,
                index.provides(),
                index.injects(),
                &consumer_counts,
                &FxHashMap::default(),
                &mut active_nodes,
                &mut expanded,
                !nodes_with_parent.contains(&file_id),
            )
        })
        .collect();

    ProvideInjectTree { roots }
}

#[expect(clippy::too_many_arguments, reason = "independent emitter inputs")]
fn build_node(
    file_id: FileId,
    registry: &ModuleRegistry,
    child_map: &FxHashMap<FileId, Vec<FileId>>,
    provides_map: &FxHashMap<FileId, Vec<ProvideEntry>>,
    injects_map: &FxHashMap<FileId, Vec<InjectEntry>>,
    consumer_counts: &FxHashMap<(FileId, u32), usize>,
    active_providers: &FxHashMap<CompactString, FileId>,
    active_nodes: &mut FxHashSet<FileId>,
    expanded: &mut FxHashSet<(FileId, Vec<(CompactString, FileId)>)>,
    show_injects: bool,
) -> ProvideNode {
    active_nodes.insert(file_id);

    let component_name = registry.get(file_id).and_then(|e| e.component_name.clone());

    // Build provides info
    let provides: Vec<ProvideInfo> = provides_map
        .get(&file_id)
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let key = match &p.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let count = *consumer_counts.get(&(file_id, p.start)).unwrap_or(&0);
                    ProvideInfo {
                        key,
                        value_type: p.value_type.clone(),
                        offset: p.start,
                        consumer_count: count,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Build injects info
    let injects = injects_map
        .get(&file_id)
        .filter(|_| show_injects)
        .map(|is| {
            is.iter()
                .map(|i| {
                    let key = match &i.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let key_identity = provide_key_identity(&i.key);
                    let provider = active_providers.get(&key_identity).copied();
                    InjectInfo {
                        key,
                        has_default: i.default_value.is_some(),
                        provider,
                        offset: i.start,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut child_providers = active_providers.clone();
    if let Some(provides) = provides_map.get(&file_id) {
        for provide in provides {
            child_providers.insert(provide_key_identity(&provide.key), file_id);
        }
    }
    let mut provider_context = child_providers
        .iter()
        .map(|(key, provider)| (key.clone(), *provider))
        .collect::<Vec<_>>();
    provider_context.sort_by(|left, right| left.0.cmp(&right.0));

    // A shared DAG node needs one expanded subtree per provider context.
    // Repeated render paths in the same context retain the node but refer to
    // the already-expanded descendants instead of copying them exponentially.
    let mut children = Vec::new();
    if expanded.insert((file_id, provider_context))
        && let Some(child_ids) = child_map.get(&file_id)
    {
        for &child_id in child_ids {
            if active_nodes.contains(&child_id) {
                continue;
            }
            let child_node = build_node(
                child_id,
                registry,
                child_map,
                provides_map,
                injects_map,
                consumer_counts,
                &child_providers,
                active_nodes,
                expanded,
                true,
            );
            children.push(child_node);
        }
    }

    active_nodes.remove(&file_id);

    ProvideNode {
        file_id,
        component_name,
        provides,
        injects,
        children,
    }
}

fn select_root_ids(
    index: &ProvideInjectIndex,
    included_nodes: &FxHashSet<FileId>,
    nodes_with_parent: &FxHashSet<FileId>,
    child_map: &FxHashMap<FileId, Vec<FileId>>,
    branches: &[ProvideInjectBranch],
) -> Vec<FileId> {
    let mut roots = included_nodes
        .iter()
        .copied()
        .filter(|file_id| !nodes_with_parent.contains(file_id))
        .collect::<Vec<_>>();
    let mut covered = FxHashSet::default();
    for &root in &roots {
        mark_reachable(root, child_map, &mut covered);
    }

    let mut cyclic_starts = branches
        .iter()
        .filter_map(|branch| branch.path.first().copied())
        .filter(|file_id| !covered.contains(file_id))
        .collect::<Vec<_>>();
    cyclic_starts.sort_by(|left, right| index.compare_files(*left, *right));
    cyclic_starts.dedup();
    roots.extend(cyclic_starts.iter().copied());
    for root in cyclic_starts {
        mark_reachable(root, child_map, &mut covered);
    }

    while let Some(root) = included_nodes
        .iter()
        .copied()
        .filter(|file_id| !covered.contains(file_id))
        .min_by(|left, right| index.compare_files(*left, *right))
    {
        roots.push(root);
        mark_reachable(root, child_map, &mut covered);
    }

    roots.sort_by(|left, right| index.compare_files(*left, *right));
    roots.dedup();
    roots
}

fn mark_reachable(
    root: FileId,
    child_map: &FxHashMap<FileId, Vec<FileId>>,
    covered: &mut FxHashSet<FileId>,
) {
    let mut pending = vec![root];
    while let Some(file_id) = pending.pop() {
        if !covered.insert(file_id) {
            continue;
        }
        if let Some(children) = child_map.get(&file_id) {
            pending.extend(children.iter().copied());
        }
    }
}
