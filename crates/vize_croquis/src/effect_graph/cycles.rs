use super::EffectEdge;
use std::collections::{BTreeMap, BTreeSet};
use vize_carton::CompactString;

pub(super) fn cycle_summary(edges: &[EffectEdge]) -> (usize, usize) {
    let mut forward: BTreeMap<CompactString, Vec<CompactString>> = BTreeMap::new();
    let mut reverse: BTreeMap<CompactString, Vec<CompactString>> = BTreeMap::new();
    let mut nodes = BTreeSet::new();
    for edge in edges {
        nodes.insert(edge.from.clone());
        nodes.insert(edge.to.clone());
        forward
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        reverse
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }

    let mut visited = BTreeSet::new();
    let mut finish_order = Vec::with_capacity(nodes.len());
    for node in &nodes {
        if visited.contains(node) {
            continue;
        }
        let mut pending = vec![(node.clone(), false)];
        while let Some((current, expanded)) = pending.pop() {
            if expanded {
                finish_order.push(current);
                continue;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            pending.push((current.clone(), true));
            if let Some(children) = forward.get(&current) {
                pending.extend(
                    children
                        .iter()
                        .rev()
                        .filter(|child| !visited.contains(*child))
                        .cloned()
                        .map(|child| (child, false)),
                );
            }
        }
    }

    let mut assigned = BTreeSet::new();
    let mut cycle_count = 0usize;
    let mut cycle_node_count = 0usize;
    while let Some(node) = finish_order.pop() {
        if !assigned.insert(node.clone()) {
            continue;
        }
        let mut component = vec![node.clone()];
        let mut pending = vec![node];
        while let Some(current) = pending.pop() {
            for parent in reverse.get(&current).into_iter().flatten() {
                if assigned.insert(parent.clone()) {
                    component.push(parent.clone());
                    pending.push(parent.clone());
                }
            }
        }

        let cyclic = component.len() > 1
            || component.iter().any(|member| {
                forward
                    .get(member)
                    .is_some_and(|children| children.contains(member))
            });
        if cyclic {
            cycle_count = cycle_count.saturating_add(1);
            cycle_node_count = cycle_node_count.saturating_add(component.len());
        }
    }

    (cycle_count, cycle_node_count)
}
