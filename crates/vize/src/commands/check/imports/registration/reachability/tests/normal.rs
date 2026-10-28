use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use super::super::scan_route_reachability;
use super::{CanonicalPathCache, ImportFileOptions, PathAliasResolver, route};
use crate::commands::check::imports::session::{AliasScope, LocalImportSession};

struct State {
    session: LocalImportSession,
    canonical: CanonicalPathCache,
    packages: vize_canon::PackageRouteResolver,
}

fn execute<const RECORD_INPUTS: bool>(
    routes: &[vize_canon::PackageRoute],
    aliases: &PathAliasResolver,
    states: &mut [State],
    results: &mut Vec<vize_canon::batch::PackageRouteReachability>,
) {
    let options = ImportFileOptions::from(false);
    let scope = AliasScope::new(Some(aliases), options);
    for (state, route) in states.iter_mut().zip(routes.iter().cycle()) {
        let sources = state
            .session
            .registration_cache
            .reachability_sources(scope.edges());
        sources.set_alias_scope(scope.token());
        // Exactly the production untracked caller: the scan interns then clears
        // local histories, and retains AliasProbe answers in its live sources.
        let answer = scan_route_reachability::<RECORD_INPUTS>(
            route,
            sources,
            &mut state.canonical,
            options,
            Some(aliases),
            &mut state.packages,
        );
        answer.record_work(&mut state.packages);
        results.push(answer);
    }
}

#[test]
#[ignore = "manual optimized unprofiled source carrier; requires a sealed physical fixture"]
fn normal_local_probe_histories() {
    assert!(
        !cfg!(feature = "profiling"),
        "normal operation must not use profiling"
    );
    assert!(
        !cfg!(debug_assertions),
        "normal operation requires optimized release assertions"
    );
    let role = std::env::var("VIZE_PROBE_NORMAL_ROLE").unwrap();
    assert!(role == "legacy" || role == "candidate");
    let root = PathBuf::from(std::env::var_os("VIZE_PROBE_FIXTURE").unwrap())
        .canonicalize()
        .unwrap();
    let group = std::env::var("VIZE_PROBE_NORMAL_GROUP").unwrap();
    assert!(matches!(group.as_str(), "relative" | "absolute" | "mixed"));
    let imports: usize = std::env::var("VIZE_PROBE_NORMAL_IMPORTS")
        .unwrap()
        .parse()
        .unwrap();
    assert!([10, 100, 1000].contains(&imports));
    let output = PathBuf::from(std::env::var_os("VIZE_PROBE_NORMAL_OUTPUT").unwrap());
    assert!(!output.exists());
    let group_root = root.join(&group).join(imports.to_string());
    let route_count = imports.div_ceil(50);
    let repeats = 10_000_usize.div_ceil(imports);
    let routes = (0..route_count)
        .map(|index| {
            let package = group_root.join(format!("p{index:04}"));
            route(&package, package.join("entry.ts"))
        })
        .collect::<Vec<_>>();
    let aliases = PathAliasResolver::from_tsconfig(Some(&group_root.join("tsconfig.json")));
    let mut states = (0..route_count * repeats)
        .map(|_| {
            let mut packages = vize_canon::PackageRouteResolver::default();
            let session = LocalImportSession::new(&mut packages);
            State {
                session,
                canonical: CanonicalPathCache::default(),
                packages,
            }
        })
        .collect::<Vec<_>>();
    let mut results = Vec::with_capacity(states.len());
    // Fixture preparation, aliases, sessions, report capacity and source hashing
    // are outside the interval. There is no observer allocator or instrumentation.
    let start = Instant::now();
    if role == "legacy" {
        execute::<true>(&routes, &aliases, &mut states, &mut results);
    } else {
        execute::<false>(&routes, &aliases, &mut states, &mut results);
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let mut files = 0;
    let mut bytes = 0;
    let mut edges = 0;
    let mut parses = 0;
    for (result, state) in results.iter().zip(&states) {
        assert_eq!(
            result.outcome,
            vize_canon::batch::ReachabilityOutcome::DoesNotReachVue
        );
        assert!(result.inputs.is_empty());
        assert_eq!(state.session.registration_cache.finished_scans(), 1);
        assert_eq!(state.packages.metrics().reachability_checks, 1);
        files += result.work.files;
        bytes += result.work.bytes;
        edges += result.work.edges;
        parses += result.work.parses;
    }
    assert_eq!(
        edges,
        imports * repeats,
        "every prepared import was actually traversed"
    );
    assert_eq!(files, parses, "every popped source was actually parsed");
    assert!(files >= route_count * repeats);
    std::hint::black_box((files, bytes, edges, parses));
    let receipt = serde_json::json!({
        "schema": "vize.cli-probe-history.normal-source", "version": 1,
        "role": role, "group": group, "imports": imports,
        "fixtureRootPhysical": root, "routeCount": route_count, "repeats": repeats,
        "compiledSourceRevision": option_env!("VIZE_PROBE_NORMAL_SOURCE_SHA"),
        "elapsedNs": elapsed_ns, "files": files, "bytes": bytes,
        "edges": edges, "parses": parses, "scans": results.len(),
        "normalOptimizedSourceOperation": true,
        "profilingFeature": false, "normalHistoricalBinaryComparison": false,
        "coldQualified": false, "wholeCliPerformanceQualified": false,
        "peakLiveBytes": null, "processTreeRss": null, "hardwareInstructions": null,
        "methodology": "one OPT3 test image; legacy consttrue preserves old source body; discarded histories are consumed only by the actual untracked source interner, never blackboxed or reported; alias proof caches remain live until after the timer",
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_vec_pretty(&receipt).unwrap().as_slice())
        .unwrap();
}
