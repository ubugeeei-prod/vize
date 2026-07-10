use super::index::ProvideInjectIndex;
use super::keys::provide_key_identity;
use super::types::{
    InjectInfo, ProvideInfo, ProvideInjectBranch, ProvideInjectMatch, ProvideInjectTree,
    ProvideNode,
};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

type BranchesByInject<'a> = FxHashMap<(FileId, CompactString, u32), Vec<&'a ProvideInjectBranch>>;

#[allow(dead_code)]
pub fn build_provide_inject_tree(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    _matches: &[ProvideInjectMatch],
) -> ProvideInjectTree {
    let index = ProvideInjectIndex::new(registry, graph);
    let (_, branches, _) = super::analysis::analyze_provide_inject_with_index(&index);
    build_provide_inject_tree_with_index(registry, &index, &branches)
}

pub(crate) fn build_provide_inject_tree_with_index(
    registry: &ModuleRegistry,
    index: &ProvideInjectIndex,
    branches: &[ProvideInjectBranch],
) -> ProvideInjectTree {
    let mut consumer_counts: FxHashMap<(FileId, u32), usize> = FxHashMap::default();
    let mut branches_by_inject = BranchesByInject::default();

    for branch in branches {
        if let (Some(provider), Some(provide_offset)) = (branch.provider, branch.provide_offset) {
            *consumer_counts
                .entry((provider, provide_offset))
                .or_insert(0) += 1;
        }
        branches_by_inject
            .entry((
                branch.consumer,
                branch.key_identity.clone(),
                branch.inject_offset,
            ))
            .or_default()
            .push(branch);
    }

    // Build the displayed tree from both matched and terminal unmatched paths.
    // This keeps pass-through components visible even when they do not provide
    // or inject the key themselves.
    let mut included_nodes = FxHashSet::default();
    let mut child_map: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
    let mut nodes_with_parent = FxHashSet::default();

    for branch in branches {
        for file_id in &branch.path {
            included_nodes.insert(*file_id);
        }
        for pair in branch.path.windows(2) {
            let parent = pair[0];
            let child = pair[1];
            child_map.entry(parent).or_default().push(child);
            nodes_with_parent.insert(child);
        }
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
            let mut ancestors = Vec::new();
            build_node(
                file_id,
                registry,
                &child_map,
                index.provides(),
                index.injects(),
                &consumer_counts,
                &branches_by_inject,
                &mut ancestors,
            )
        })
        .collect();

    ProvideInjectTree { roots }
}

#[allow(unused, clippy::too_many_arguments)]
fn build_node(
    file_id: FileId,
    registry: &ModuleRegistry,
    child_map: &FxHashMap<FileId, Vec<FileId>>,
    provides_map: &FxHashMap<FileId, Vec<ProvideEntry>>,
    injects_map: &FxHashMap<FileId, Vec<InjectEntry>>,
    consumer_counts: &FxHashMap<(FileId, u32), usize>,
    branches_by_inject: &BranchesByInject<'_>,
    ancestors: &mut Vec<FileId>,
) -> ProvideNode {
    ancestors.push(file_id);

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
        .map(|is| {
            is.iter()
                .filter_map(|i| {
                    let key = match &i.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let key_identity = provide_key_identity(&i.key);
                    // A reused component can occur below different providers.
                    // Resolve against this rendered ancestor branch, not only
                    // the consumer file and key shared by every occurrence.
                    let provider = branches_by_inject
                        .get(&(file_id, key_identity, i.start))
                        .and_then(|branches| provider_for_branch(branches, ancestors))?;
                    Some(InjectInfo {
                        key,
                        has_default: i.default_value.is_some(),
                        provider,
                        offset: i.start,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Find children (components that inject from this provider)
    let mut children = Vec::new();
    if let Some(child_ids) = child_map.get(&file_id) {
        for &child_id in child_ids {
            if ancestors.contains(&child_id) {
                continue;
            }
            let child_node = build_node(
                child_id,
                registry,
                child_map,
                provides_map,
                injects_map,
                consumer_counts,
                branches_by_inject,
                ancestors,
            );
            children.push(child_node);
        }
    }

    ancestors.pop();

    ProvideNode {
        file_id,
        component_name,
        provides,
        injects,
        children,
    }
}

fn provider_for_branch(
    branches: &[&ProvideInjectBranch],
    ancestors: &[FileId],
) -> Option<Option<FileId>> {
    branches
        .iter()
        .filter(|branch| ancestors.ends_with(&branch.path))
        .max_by_key(|branch| branch.path.len())
        .map(|branch| branch.provider)
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
