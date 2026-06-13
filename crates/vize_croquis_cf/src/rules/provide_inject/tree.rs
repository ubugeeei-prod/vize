use super::index::ProvideInjectIndex;
use super::keys::provide_key_identity;
use super::types::{InjectInfo, ProvideInfo, ProvideInjectMatch, ProvideInjectTree, ProvideNode};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

#[allow(dead_code)]
pub fn build_provide_inject_tree(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    matches: &[ProvideInjectMatch],
) -> ProvideInjectTree {
    let index = ProvideInjectIndex::new(registry, graph);
    build_provide_inject_tree_with_index(registry, &index, matches)
}

pub(crate) fn build_provide_inject_tree_with_index(
    registry: &ModuleRegistry,
    index: &ProvideInjectIndex,
    matches: &[ProvideInjectMatch],
) -> ProvideInjectTree {
    let mut consumer_counts: FxHashMap<(FileId, CompactString), usize> = FxHashMap::default();
    let mut provider_by_consumer_key: FxHashMap<(FileId, CompactString), FileId> =
        FxHashMap::default();

    // Count consumers for each provide
    for m in matches {
        *consumer_counts
            .entry((m.provider, m.key_identity.clone()))
            .or_insert(0) += 1;
        provider_by_consumer_key
            .entry((m.consumer, m.key_identity.clone()))
            .or_insert(m.provider);
    }

    // Build the displayed tree from resolved provider -> ... -> consumer paths.
    // This keeps pass-through components visible even when they do not provide
    // or inject the key themselves.
    let mut included_nodes = FxHashSet::default();
    let mut child_map: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
    let mut parent_map: FxHashMap<FileId, FileId> = FxHashMap::default();

    for m in matches {
        for file_id in &m.path {
            included_nodes.insert(*file_id);
        }
        for pair in m.path.windows(2) {
            let parent = pair[0];
            let child = pair[1];
            child_map.entry(parent).or_default().push(child);
            parent_map.entry(child).or_insert(parent);
        }
    }

    for &file_id in index.provides().keys() {
        included_nodes.insert(file_id);
    }
    for &file_id in index.injects().keys() {
        included_nodes.insert(file_id);
    }

    for children in child_map.values_mut() {
        children.sort_by_key(|id| id.as_u32());
        children.dedup();
    }

    let mut root_ids: Vec<_> = included_nodes
        .iter()
        .copied()
        .filter(|file_id| !parent_map.contains_key(file_id))
        .collect();
    root_ids.sort_by_key(|id| id.as_u32());

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
                &provider_by_consumer_key,
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
    consumer_counts: &FxHashMap<(FileId, CompactString), usize>,
    provider_by_consumer_key: &FxHashMap<(FileId, CompactString), FileId>,
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
                    let key_identity = provide_key_identity(&p.key);
                    let count = *consumer_counts.get(&(file_id, key_identity)).unwrap_or(&0);
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
                .map(|i| {
                    let key = match &i.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let key_identity = provide_key_identity(&i.key);
                    let provider = provider_by_consumer_key
                        .get(&(file_id, key_identity))
                        .copied();
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
                provider_by_consumer_key,
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
