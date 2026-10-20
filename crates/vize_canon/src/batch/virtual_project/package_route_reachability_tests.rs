#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};

use super::{
    ReachabilityBudget, ReachabilityOutcome, package_route_reaches_vue_with_budget,
    scan_package_route_reachability_with_budget,
};

const SOURCE_OPTIONS: crate::PackageSourceOptions = crate::PackageSourceOptions::new(true, true);

#[test]
fn cycles_and_duplicate_edges_have_exact_unique_work() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "import './a'\nimport './a'\n");
    fixture.write("a.ts", "export * from './b'\nexport * from './b'\n");
    fixture.write("b.ts", "export * from './a'\n");

    let result = fixture.reachability("entry.ts", generous_budget());

    assert_eq!(result.outcome, ReachabilityOutcome::DoesNotReachVue);
    assert_eq!(result.work.files, 3);
    assert_eq!(result.work.parses, 3);
    assert_eq!(result.work.edges, 3);
    assert_eq!(result.work.packages, 1);
}

#[test]
fn scans_sharing_source_facts_match_fresh_scans_and_read_each_file_once() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "import './a'\nimport './b'\n");
    fixture.write("a.ts", "export * from './b'\n");
    fixture.write(
        "b.ts",
        &format!("export const big = '{}';\n", "x".repeat(200)),
    );
    let budget = ReachabilityBudget {
        max_file_bytes: 128,
        ..generous_budget()
    };
    let route = route(
        &fixture.package_root,
        &fixture.package_root.join("entry.ts"),
    );
    let fresh = fixture.reachability_for_route(&route, budget);
    let mut sources = super::ReachabilitySources::default();
    let mut scan = |sources: &mut super::ReachabilitySources| {
        package_route_reaches_vue_with_budget(
            &route,
            &[],
            &super::super::package_resolution::PackageResolutionSettings::default(),
            &mut crate::PackageRouteResolver::default(),
            SOURCE_OPTIONS,
            sources,
            budget,
        )
    };
    let first = scan(&mut sources);
    // The pass treats the filesystem as stable: a second scan reuses facts.
    fixture.write("a.ts", "export {}\n");
    let second = scan(&mut sources);

    assert_eq!(fresh.outcome, ReachabilityOutcome::BudgetExceeded);
    for result in [&first, &second] {
        assert_eq!(result.outcome, fresh.outcome);
        assert_eq!(result.work, fresh.work);
        assert_eq!(result.inputs, fresh.inputs);
    }
}

#[test]
fn scans_sharing_sources_resolve_path_and_package_edges_once() {
    let fixture = Fixture::new();
    fixture.write("one.ts", "import './shared'\nimport 'dep'\n");
    fixture.write("two.ts", "import './shared'\n");
    fixture.write("shared.ts", "import './leaf'\nimport 'dep'\n");
    fixture.write("leaf.ts", "export {}\n");
    let calls = std::cell::RefCell::new(Vec::new());
    let scan = |entry: &str, sources: &mut super::ReachabilitySources| {
        scan_package_route_reachability_with_budget(
            &route(&fixture.package_root, &fixture.package_root.join(entry)),
            sources,
            |importer, specifier| {
                calls.borrow_mut().push(format!("local {specifier}"));
                let base = importer.parent().unwrap().join(specifier);
                let candidate = base.with_extension("ts");
                let consulted = vec![candidate.clone()];
                if specifier.starts_with('.') && candidate.is_file() {
                    (Some(candidate), consulted)
                } else {
                    (None, Vec::new())
                }
            },
            |_, specifier, _| {
                calls.borrow_mut().push(format!("package {specifier}"));
                (
                    None,
                    vec![fixture.package_root.join("node_modules").join(specifier)],
                )
            },
            generous_budget(),
        )
    };
    let fresh = ["one.ts", "two.ts"].map(|entry| scan(entry, &mut Default::default()));
    calls.borrow_mut().clear();
    let mut sources = super::ReachabilitySources::default();
    let shared = ["one.ts", "two.ts"].map(|entry| scan(entry, &mut sources));

    for (fresh, shared) in fresh.iter().zip(&shared) {
        assert_eq!(shared.outcome, fresh.outcome);
        assert_eq!(shared.work, fresh.work);
        assert_eq!(shared.inputs, fresh.inputs);
    }
    // `shared.ts` resolves `./leaf` and the `dep` package once. Aliases may
    // resolve against the scanned route, so only they repeat.
    let mut calls = calls.into_inner();
    calls.sort();
    assert_eq!(
        calls,
        [
            "local ./leaf",
            "local ./shared",
            "local ./shared",
            "local dep",
            "local dep",
            "local dep",
            "package dep",
            "package dep",
        ]
    );
}

#[test]
fn long_cyclic_duplicate_graph_stops_at_exact_unique_work_bound() {
    let fixture = Fixture::new();
    let mut expected_bytes = 0;
    for index in 0_usize..140 {
        let previous = index.saturating_sub(1);
        let next = index + 1;
        let content = if index == 0 {
            format!("export * from './f{next}'\nexport * from './f{next}'\n")
        } else {
            format!(
                "export * from './f{previous}'\nexport * from './f{next}'\nexport * from './f{next}'\n"
            )
        };
        if index < 64 {
            expected_bytes += content.len();
        }
        fixture.write(&format!("f{index}.ts"), &content);
    }
    let budget = ReachabilityBudget {
        max_files: 64,
        max_edges: 512,
        max_parses: 64,
        ..generous_budget()
    };

    let result = fixture.reachability("f0.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert_eq!(result.work.files, 64);
    assert_eq!(result.work.bytes, expected_bytes);
    assert_eq!(result.work.edges, 127);
    assert_eq!(result.work.parses, 64);
    assert_eq!(result.work.packages, 1);
    assert!(result.inputs.iter().any(|path| path.ends_with("f64.ts")));
}

#[test]
fn vue_target_at_the_file_boundary_is_reachable() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "export * from './a'\n");
    fixture.write("a.ts", "export { default } from './Widget.vue'\n");
    fixture.write("Widget.vue", "<template />\n");
    let budget = ReachabilityBudget {
        max_files: 2,
        ..generous_budget()
    };

    let result = fixture.reachability("entry.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::ReachesVue);
    assert_eq!(result.work.files, 2);
    assert_eq!(result.work.parses, 2);
    assert_eq!(result.work.edges, 2);
    assert!(
        result
            .inputs
            .iter()
            .any(|path| path.ends_with("Widget.vue"))
    );
}

#[test]
fn vue_target_after_the_file_boundary_fails_closed() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "export * from './a'\n");
    fixture.write("a.ts", "export * from './b'\n");
    fixture.write("b.ts", "export { default } from './Widget.vue'\n");
    fixture.write("Widget.vue", "<template />\n");
    let budget = ReachabilityBudget {
        max_files: 2,
        ..generous_budget()
    };

    let result = fixture.reachability("entry.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert!(!result.requires_shadow());
    assert!(result.requires_tracking());
    assert_eq!(result.work.files, 2);
    assert_eq!(result.work.parses, 2);
    assert_eq!(result.work.edges, 2);
    assert!(result.inputs.iter().any(|path| path.ends_with("b.ts")));
}

#[test]
fn budget_retains_every_resolved_but_unvisited_local_input() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "export * from './a'\nexport * from './b'\n");
    fixture.write("a.ts", "export {}\n");
    fixture.write("b.ts", "export {}\n");
    let budget = ReachabilityBudget {
        max_files: 1,
        ..generous_budget()
    };

    let result = fixture.reachability("entry.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert!(result.inputs.iter().any(|path| path.ends_with("a.ts")));
    assert!(result.inputs.iter().any(|path| path.ends_with("b.ts")));
}

#[test]
fn missing_local_vue_candidate_is_retained_and_reaches_after_create() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "export { default } from './Missing'\n");

    let missing = fixture.reachability("entry.ts", generous_budget());

    assert_eq!(missing.outcome, ReachabilityOutcome::DoesNotReachVue);
    assert!(
        missing
            .inputs
            .iter()
            .any(|path| path.ends_with("Missing.vue"))
    );

    fixture.write("Missing.vue", "<template />\n");
    let created = fixture.reachability("entry.ts", generous_budget());
    assert_eq!(created.outcome, ReachabilityOutcome::ReachesVue);
}

#[test]
fn oversized_source_is_not_read_or_parsed() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", &"x".repeat(129));
    let budget = ReachabilityBudget {
        max_file_bytes: 128,
        max_total_bytes: 128,
        ..generous_budget()
    };

    let result = fixture.reachability("entry.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert_eq!(result.work.files, 1);
    assert_eq!(result.work.bytes, 0);
    assert_eq!(result.work.parses, 0);
    assert!(result.inputs.iter().any(|path| path.ends_with("entry.ts")));
}

#[test]
fn declaration_candidate_reaches_vue_before_oversized_runtime_bundle() {
    let fixture = Fixture::new();
    fixture.write("aaa.d.ts", "export { default } from './Widget.vue'\n");
    fixture.write("zzz.js", &"x".repeat(129));
    fixture.write("Widget.vue", "<template />\n");
    let budget = ReachabilityBudget {
        max_file_bytes: 128,
        max_total_bytes: 128,
        ..generous_budget()
    };
    let route = route_with_entries(
        &fixture.package_root,
        &[
            fixture.package_root.join("zzz.js"),
            fixture.package_root.join("aaa.d.ts"),
        ],
    );

    let result = fixture.reachability_for_route(&route, budget);

    assert_eq!(result.outcome, ReachabilityOutcome::ReachesVue);
    assert_eq!(result.work.files, 1);
    assert_eq!(result.work.parses, 1);
    assert_eq!(result.work.edges, 1);
}

#[test]
fn unique_edge_budget_deduplicates_before_stopping() {
    let fixture = Fixture::new();
    fixture.write(
        "entry.ts",
        "import './a'\nimport './a'\nimport './b'\nimport './c'\n",
    );
    for name in ["a.ts", "b.ts", "c.ts"] {
        fixture.write(name, "export {}\n");
    }
    let budget = ReachabilityBudget {
        max_edges: 2,
        ..generous_budget()
    };

    let result = fixture.reachability("entry.ts", budget);

    assert_eq!(result.outcome, ReachabilityOutcome::BudgetExceeded);
    assert_eq!(result.work.files, 1);
    assert_eq!(result.work.edges, 2);
    assert_eq!(result.work.parses, 1);
}

#[test]
fn resolver_metrics_expose_the_exact_last_budgeted_work() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", &"x".repeat(129));
    let route = route(
        &fixture.package_root,
        &fixture.package_root.join("entry.ts"),
    );
    let budget = ReachabilityBudget {
        max_file_bytes: 128,
        max_total_bytes: 128,
        ..generous_budget()
    };
    let mut resolver = crate::PackageRouteResolver::default();
    let result = package_route_reaches_vue_with_budget(
        &route,
        &[],
        &super::super::package_resolution::PackageResolutionSettings::default(),
        &mut resolver,
        SOURCE_OPTIONS,
        &mut Default::default(),
        budget,
    );
    result.record_work(&mut resolver);

    let metrics = resolver.metrics();
    assert_eq!(metrics.reachability_checks, 1);
    assert_eq!(metrics.reachability_budget_exceeded, 1);
    assert_eq!(metrics.last_reachability_files, 1);
    assert_eq!(metrics.last_reachability_bytes, 0);
    assert_eq!(metrics.last_reachability_edges, 0);
    assert_eq!(metrics.last_reachability_parses, 0);
    assert_eq!(metrics.last_reachability_packages, 1);
}

fn generous_budget() -> ReachabilityBudget {
    ReachabilityBudget {
        max_packages: 32,
        max_files: 32,
        max_queued_files: 256,
        max_file_bytes: 16 * 1024,
        max_total_bytes: 64 * 1024,
        max_edges: 64,
        max_parses: 32,
    }
}

struct Fixture {
    _root: tempfile::TempDir,
    package_root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let package_root = root.path().join("package");
        std::fs::create_dir_all(&package_root).unwrap();
        std::fs::write(
            package_root.join("package.json"),
            r#"{"name":"reachability-fixture"}"#,
        )
        .unwrap();
        Self {
            _root: root,
            package_root,
        }
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.package_root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn reachability(
        &self,
        entry: &str,
        budget: ReachabilityBudget,
    ) -> super::PackageRouteReachability {
        let route = route(&self.package_root, &self.package_root.join(entry));
        self.reachability_for_route(&route, budget)
    }

    fn reachability_for_route(
        &self,
        route: &crate::PackageRoute,
        budget: ReachabilityBudget,
    ) -> super::PackageRouteReachability {
        package_route_reaches_vue_with_budget(
            route,
            &[],
            &super::super::package_resolution::PackageResolutionSettings::default(),
            &mut crate::PackageRouteResolver::default(),
            SOURCE_OPTIONS,
            &mut Default::default(),
            budget,
        )
    }
}

fn route(package_root: &Path, entry: &Path) -> crate::PackageRoute {
    route_with_entries(package_root, &[entry.to_path_buf()])
}

fn route_with_entries(package_root: &Path, entries: &[PathBuf]) -> crate::PackageRoute {
    crate::PackageRoute {
        source_paths: entries.to_vec(),
        dependency_paths: Vec::new(),
        source_targets: Vec::new(),
        package_root: package_root.to_path_buf(),
        package_link_root: package_root.to_path_buf(),
        manifest_path: package_root.join("package.json"),
        package_name: Some("reachability-fixture".into()),
        workspace_source: false,
        nested_routes: Vec::new(),
    }
}

mod graph_tests {
    #![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
    use super::{
        Fixture, ReachabilityBudget, ReachabilityOutcome, generous_budget, route,
        route_with_entries, scan_package_route_reachability_with_budget,
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
            &mut Default::default(),
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
}
