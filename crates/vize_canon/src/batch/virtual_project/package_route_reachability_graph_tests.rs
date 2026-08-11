use super::{
    Fixture, ReachabilityBudget, ReachabilityOutcome, generous_budget, route, route_with_entries,
    scan_package_route_reachability_with_budget,
};

#[test]
fn nested_package_route_with_a_vue_source_reaches_vue() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "export { default } from 'nested-package'\n");
    let nested_root = fixture.package_root.join("node_modules/nested-package");
    std::fs::create_dir_all(&nested_root).unwrap();
    std::fs::write(nested_root.join("Widget.vue"), "<template />\n").unwrap();
    let nested = route_with_entries(&nested_root, &[nested_root.join("Widget.vue")]);
    let entry = route(
        &fixture.package_root,
        &fixture.package_root.join("entry.ts"),
    );

    let result = scan_package_route_reachability_with_budget(
        &entry,
        |_, _| (None, Vec::new()),
        |_, _, _| (Some(nested.clone()), Vec::new()),
        generous_budget(),
    );

    assert_eq!(result.outcome, ReachabilityOutcome::ReachesVue);
    assert_eq!(result.work.packages, 2);
    assert!(
        result
            .inputs
            .iter()
            .any(|path| path.ends_with("nested-package/Widget.vue"))
    );
}

#[test]
fn seeded_candidates_stop_at_the_queue_bound_before_any_file_is_read() {
    let fixture = Fixture::new();
    let entries = (0..8)
        .map(|index| {
            let name = format!("entry{index}.ts");
            fixture.write(&name, "export {}\n");
            fixture.package_root.join(name)
        })
        .collect::<Vec<_>>();
    let route = route_with_entries(&fixture.package_root, &entries);
    let budget = ReachabilityBudget {
        max_queued_files: 4,
        ..generous_budget()
    };

    let result = fixture.reachability_for_route(&route, budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert_eq!(result.work.files, 0);
    assert_eq!(result.work.parses, 0);
    assert_eq!(
        result
            .inputs
            .iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "ts"))
            .count(),
        5
    );
}
