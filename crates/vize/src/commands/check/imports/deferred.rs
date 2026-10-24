//! Complete workspace routes once per exact set of materialized source roots.

use super::{
    DeferredWorkspaceRoute, FxHashMap, FxHashSet, ImportReachabilityGraph, PackageRouteBinding,
    PathBuf,
};
use vize_l0::{profile, profiler::global_profiler};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct CompletionWork {
    requests: u64,
    eligible: u64,
    traversals: u64,
    reachable_nodes: u64,
}

pub(super) fn complete_routes(
    graph: &ImportReachabilityGraph,
    pending: Vec<DeferredWorkspaceRoute>,
    registered: &mut FxHashSet<PathBuf>,
    registrations: &mut Vec<PathBuf>,
    bindings: &mut Vec<PackageRouteBinding>,
) -> CompletionWork {
    let mut work = CompletionWork {
        requests: pending.len() as u64,
        ..Default::default()
    };
    if work.requests == 0 {
        return work;
    }
    let groups = profile!("cli.check.imports.route_grouping", {
        let materialized = graph.reaches_materialized_source();
        let mut groups: FxHashMap<Vec<PathBuf>, Vec<DeferredWorkspaceRoute>> = FxHashMap::default();
        for route in pending {
            let sources = route.route.all_source_paths();
            let roots = sources
                .into_iter()
                .filter(|path| materialized.contains(*path))
                .cloned()
                .collect::<Vec<_>>();
            if roots.is_empty() && !route.route.requires_workspace_source_shadow() {
                continue;
            }
            // all_source_paths already sorts and deduplicates. The complete
            // physical candidate set, rather than the package spelling, keeps
            // conditional exports and importer-local package versions separate.
            if !route.package_graph && registered.insert(route.importer_path.clone()) {
                registrations.push(route.importer_path.clone());
            }
            groups.entry(roots).or_default().push(route);
            work.eligible += 1;
        }
        groups
    });
    work.traversals = groups.len() as u64;
    for (roots, routes) in groups {
        let sources = profile!(
            "cli.check.imports.workspace_reachability",
            graph.reachable_files(roots)
        );
        work.reachable_nodes += sources.len() as u64;
        for pending in routes {
            let mut route = pending.route;
            let before = route.dependency_paths.len();
            route.dependency_paths.extend(
                sources
                    .iter()
                    .filter(|path| path.starts_with(&route.package_root))
                    .cloned(),
            );
            if route.dependency_paths.len() != before {
                route.dependency_paths.sort();
                route.dependency_paths.dedup();
            }
            bindings.push(PackageRouteBinding {
                importer_path: pending.importer_path,
                specifier: pending.specifier,
                occurrence_mode: pending.occurrence_mode,
                context: pending.context,
                route: Some(route),
                invalidation_paths: pending.invalidation_paths,
                shared_invalidation_paths: Vec::new(),
            });
        }
        // Only one closure remains live at a time; repeated-import savings do
        // not retain every package's transitive set for the rest of collection.
    }
    let profiler = global_profiler();
    profiler.record_counter("cli.check.imports.deferred_requests", work.requests);
    profiler.record_counter("cli.check.imports.deferred_eligible", work.eligible);
    profiler.record_counter("cli.check.imports.workspace_traversals", work.traversals);
    profiler.record_counter(
        "cli.check.imports.workspace_reachable_nodes",
        work.reachable_nodes,
    );
    work
}

#[cfg(test)]
mod tests;
