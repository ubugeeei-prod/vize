use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::PathBuf;

use vize_carton::FxHashSet;

use super::{ReachabilityBudget, ReachabilityWork, enqueue};

type PackageIdentity = (PathBuf, PathBuf, PathBuf);

pub(super) fn seed_route_graph(
    root: &crate::PackageRoute,
    packages: &mut FxHashSet<PackageIdentity>,
    queue: &mut BinaryHeap<(u8, Reverse<PathBuf>)>,
    queued: &mut FxHashSet<PathBuf>,
    inputs: &mut Vec<PathBuf>,
    work: &mut ReachabilityWork,
    budget: ReachabilityBudget,
) -> bool {
    let mut pending = vec![root];
    while let Some(route) = pending.pop() {
        let identity = (
            route.package_link_root.clone(),
            route.package_root.clone(),
            route.manifest_path.clone(),
        );
        if !packages.insert(identity) {
            continue;
        }
        inputs.extend([
            route.manifest_path.clone(),
            route.package_link_root.clone(),
            route.package_link_root.join("package.json"),
        ]);
        if work.packages == budget.max_packages {
            return false;
        }
        work.packages += 1;
        for path in route.source_paths.iter().chain(&route.dependency_paths) {
            if queued.contains(path) {
                continue;
            }
            inputs.push(path.clone());
            if queued.len() == budget.max_queued_files {
                return false;
            }
            enqueue(queue, queued, path.clone());
        }
        let mut nested = route.nested_routes.iter().collect::<Vec<_>>();
        nested.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
        pending.extend(nested.into_iter().rev());
    }
    true
}

#[cfg(test)]
#[path = "package_route_reachability_route_graph_tests.rs"]
mod tests;
