//! Finished scans outlive one alias configuration only where its answers agree.

use std::path::{Path, PathBuf};

use super::tests::{Fixture, generous_budget, route};
use super::{
    ReachabilityOutcome, ReachabilitySources, scan_package_route_reachability_with_budget,
};

/// Scan `entry.ts` with `#imports` aliased to `target`, or left unaliased.
fn scan(
    fixture: &Fixture,
    sources: &mut ReachabilitySources,
    target: Option<&str>,
) -> super::PackageRouteReachability {
    let route = route(
        &fixture.package_root,
        &fixture.package_root.join("entry.ts"),
    );
    let resolve_local = |importer: &Path, specifier: &str| {
        let resolved = if specifier == "#imports" {
            target.map(|target| fixture.package_root.join(target))
        } else {
            let candidate = importer.parent().unwrap().join(specifier);
            Some(candidate).filter(|candidate| candidate.is_file())
        };
        (resolved, Vec::<PathBuf>::new())
    };
    scan_package_route_reachability_with_budget(
        &route,
        sources,
        resolve_local,
        |_, _, _| (None, Vec::new()),
        generous_budget(),
    )
}

#[test]
fn another_scope_reuses_a_scan_only_when_its_aliases_answer_the_same() {
    let fixture = Fixture::new();
    fixture.write("entry.ts", "import '#imports'\nimport './leaf.ts'\n");
    fixture.write("leaf.ts", "export {}\n");
    fixture.write("app.ts", "import './Widget.vue'\n");
    fixture.write("Widget.vue", "<template />\n");
    fixture.write("server.ts", "export {}\n");

    let mut sources = ReachabilitySources::untracked();
    sources.set_alias_scope(1);
    let app = scan(&fixture, &mut sources, Some("app.ts"));
    assert_eq!(app.outcome, ReachabilityOutcome::ReachesVue);
    assert_eq!(sources.finished_scans(), 1);

    // Same answer under other aliases: reused.
    sources.set_alias_scope(2);
    let shared = scan(&fixture, &mut sources, Some("app.ts"));
    assert_eq!(shared.outcome, app.outcome);
    assert_eq!(shared.work, app.work);
    assert_eq!(sources.finished_scans(), 1);

    // `#imports` resolves elsewhere: scanned again, as a fresh pass would.
    sources.set_alias_scope(3);
    let server = scan(&fixture, &mut sources, Some("server.ts"));
    let fresh = scan(
        &fixture,
        &mut ReachabilitySources::untracked(),
        Some("server.ts"),
    );
    assert_eq!(server.outcome, ReachabilityOutcome::DoesNotReachVue);
    assert_eq!(server.outcome, fresh.outcome);
    assert_eq!(server.work, fresh.work);
    assert_eq!(sources.finished_scans(), 2);

    // An unaliased spelling is an answer too.
    sources.set_alias_scope(4);
    let unaliased = scan(&fixture, &mut sources, None);
    assert_eq!(unaliased.outcome, ReachabilityOutcome::DoesNotReachVue);
    assert_eq!(sources.finished_scans(), 3);

    // Each scope keeps its verified scan.
    sources.set_alias_scope(2);
    assert_eq!(
        scan(&fixture, &mut sources, Some("app.ts")).outcome,
        ReachabilityOutcome::ReachesVue
    );
    sources.set_alias_scope(3);
    assert_eq!(
        scan(&fixture, &mut sources, Some("server.ts")).outcome,
        ReachabilityOutcome::DoesNotReachVue
    );
    assert_eq!(sources.finished_scans(), 3);
}
