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
        |_, _| (None, Vec::new()),
        |_, _, _| (None, Vec::new()),
        budget,
    );

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert_eq!(result.work.packages, 2);
    assert_eq!(result.work.files, 0);
    assert!(
        result
            .inputs
            .iter()
            .any(|path| path.ends_with("p2/package.json"))
    );
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
