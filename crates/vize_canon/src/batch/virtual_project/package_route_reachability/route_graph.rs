use std::collections::BinaryHeap;
use std::path::PathBuf;

use vize_carton::FxHashSet;

use super::{QueueEntry, ReachabilityBudget, ReachabilityWork, enqueue};

type PackageIdentity = (PathBuf, PathBuf, PathBuf);

/// Queue a route's package graph at `depth` package hops. `None` means the
/// budget stopped the seed; `Some(true)` that a `.vue` source was queued.
#[expect(clippy::too_many_arguments, reason = "one scan's mutable state")]
pub(super) fn seed_route_graph(
    root: &crate::PackageRoute,
    depth: u8,
    packages: &mut FxHashSet<PackageIdentity>,
    queue: &mut BinaryHeap<QueueEntry>,
    queued: &mut FxHashSet<PathBuf>,
    inputs: &mut super::InputSet,
    work: &mut ReachabilityWork,
    budget: ReachabilityBudget,
) -> Option<bool> {
    let mut queued_vue = false;
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
        for path in [
            &route.manifest_path,
            &route.package_link_root,
            &route.package_link_root.join("package.json"),
        ] {
            super::record_input(inputs, path);
        }
        if work.packages == budget.max_packages {
            return None;
        }
        work.packages += 1;
        for path in route.source_paths.iter().chain(&route.dependency_paths) {
            if queued.contains(path) {
                continue;
            }
            super::record_input(inputs, path);
            if queued.len() == budget.max_queued_files {
                return None;
            }
            queued_vue |= enqueue(queue, queued, path.clone(), depth);
        }
        let mut nested = route.nested_routes.iter().collect::<Vec<_>>();
        nested.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
        pending.extend(nested.into_iter().rev());
    }
    Some(queued_vue)
}

#[cfg(test)]
mod tests {
    #![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
    use std::path::Path;

    use super::super::{
        ReachabilityBudget, ReachabilityOutcome, scan_package_route_reachability_with_budget,
    };

    #[test]
    fn nested_package_graph_deduplicates_identity_and_stops_before_expansion() {
        let root = tempfile::tempdir().unwrap();
        let package_two = route(root.path(), 2, Vec::new());
        let package_one = route(root.path(), 1, vec![package_two]);
        let duplicate_one = route(root.path(), 1, Vec::new());
        let package_zero = route(root.path(), 0, vec![package_one, duplicate_one]);
        let budget = ReachabilityBudget {
            max_packages: 2,
            max_files: 8,
            max_queued_files: 64,
            max_file_bytes: 1024,
            max_total_bytes: 1024,
            max_edges: 8,
            max_parses: 8,
        };

        let result = scan_package_route_reachability_with_budget(
            &package_zero,
            &mut Default::default(),
            |_, _| (None, Vec::new()),
            |_, _, _| (None, Vec::new()),
            budget,
        );

        assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
        assert_eq!(result.work.packages, 2);
        assert_eq!(result.work.files, 0);
        let inputs = result
            .inputs
            .iter()
            .map(|path| path.strip_prefix(root.path()).unwrap().to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            inputs,
            [
                "p0",
                "p0/package.json",
                "p1",
                "p1/package.json",
                "p2",
                "p2/package.json"
            ]
        );
    }

    #[test]
    fn importer_package_graph_is_scanned_before_dependency_chunks() {
        // The dependency's declaration chunks sort before the importer's own
        // barrel, and together exceed the byte budget: path order alone spent
        // the budget there and never reached the importer's local `.vue`.
        let root = tempfile::tempdir().unwrap();
        let write = |relative: &str, content: &str| {
            let path = root.path().join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, content).unwrap();
            path
        };
        let entry = write(
            "p0/src/index.ts",
            "export * from './components';\nimport { x } from 'dep';\n",
        );
        let barrel = write(
            "p0/src/components/index.ts",
            "export { default } from './a.vue';\n",
        );
        let vue = write("p0/src/components/a.vue", "<template />\n");
        let chunk = "export declare const value: number;\n".repeat(4);
        let chunks = ["a-dep/c1.d.ts", "a-dep/c2.d.ts", "a-dep/c3.d.ts"]
            .map(|relative| write(relative, &chunk));
        let mut dependency = route(root.path(), 9, Vec::new());
        dependency.package_root = root.path().join("a-dep");
        dependency.package_link_root = dependency.package_root.clone();
        dependency.manifest_path = dependency.package_root.join("package.json");
        dependency.source_paths = chunks.to_vec();
        let mut importer = route(root.path(), 0, Vec::new());
        importer.source_paths = vec![entry];
        let budget = ReachabilityBudget {
            max_packages: 8,
            max_files: 8,
            max_queued_files: 64,
            max_file_bytes: 1024,
            max_total_bytes: chunk.len() * 2,
            max_edges: 8,
            max_parses: 8,
        };

        let result = scan_package_route_reachability_with_budget(
            &importer,
            &mut Default::default(),
            |_, specifier| match specifier {
                "./components" => (Some(barrel.clone()), Vec::new()),
                "./a.vue" => (Some(vue.clone()), Vec::new()),
                _ => (None, Vec::new()),
            },
            |_, specifier, _| ((specifier == "dep").then(|| dependency.clone()), Vec::new()),
            budget,
        );

        assert_eq!(result.outcome, ReachabilityOutcome::ReachesVue);
        assert_eq!(result.work.files, 2);
    }

    fn route(
        root: &Path,
        index: usize,
        nested_routes: Vec<crate::PackageRoute>,
    ) -> crate::PackageRoute {
        let package_root = root.join(format!("p{index}"));
        crate::PackageRoute {
            source_paths: Vec::new(),
            dependency_paths: Vec::new(),
            source_targets: Vec::new(),
            package_root: package_root.clone(),
            package_link_root: package_root.clone(),
            manifest_path: package_root.join("package.json"),
            package_name: Some(format!("p{index}").into()),
            workspace_source: false,
            nested_routes,
        }
    }
}
