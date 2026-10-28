use super::Bins;
use vize_canon::batch::VirtualFile;
use vize_carton::FxHashMap;

pub(super) fn pack(
    partitioned: &[&VirtualFile],
    component_files: FxHashMap<usize, Vec<usize>>,
    servers: usize,
) -> Option<Bins> {
    let weight = |file_indices: &[usize]| -> usize {
        file_indices
            .iter()
            .filter_map(|&index| partitioned.get(index))
            .map(|file| file.content.len())
            .sum()
    };
    if component_files.len() < 2 {
        return None;
    }
    // Evaluate each member once. Sorting must not re-walk a component for
    // every comparison, especially when a large graph meets many small ones.
    let mut component_groups: Vec<(usize, Vec<usize>)> = component_files
        .into_values()
        .map(|files| (weight(&files), files))
        .collect();
    let total_weight: usize = component_groups.iter().map(|(weight, _)| *weight).sum();
    component_groups.sort_by(|(left_weight, left), (right_weight, right)| {
        right_weight
            .cmp(left_weight)
            .then_with(|| left.first().cmp(&right.first()))
    });

    let servers = servers.min(component_groups.len());
    let mut bins: Vec<(usize, Vec<usize>)> = vec![(0, Vec::new()); servers];
    for (group_weight, group) in component_groups {
        let Some(bin) = bins.iter_mut().min_by_key(|(bin_weight, _)| *bin_weight) else {
            return None;
        };
        bin.0 += group_weight;
        bin.1.extend(group);
    }
    let largest = bins.iter().map(|(bin_weight, _)| *bin_weight).max();
    // Wall time tracks the heaviest shard; below ~25% savings the duplicated
    // per-program work on shared and ambient sources outweighs the win.
    if largest.unwrap_or(0) * 4 >= total_weight * 3 {
        return None;
    }

    Some(bins)
}
