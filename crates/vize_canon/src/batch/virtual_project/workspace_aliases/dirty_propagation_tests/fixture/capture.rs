//! Entire deterministic typed-state observations, never a text-edit certificate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::path::PathBuf;

use vize_carton::{FxHashMap, FxHashSet, String, cstr};

use super::{Graph, Snapshot, history, snapshot, work};

fn ordered<K: Ord, V>(map: &FxHashMap<K, V>) -> BTreeMap<&K, &V> {
    map.iter().collect()
}

fn sets<K: Ord, V: Ord>(map: &FxHashMap<K, FxHashSet<V>>) -> BTreeMap<&K, BTreeSet<&V>> {
    map.iter()
        .map(|(key, values)| (key, values.iter().collect()))
        .collect()
}

fn text(value: impl Debug) -> String {
    cstr!("{value:#?}")
}

fn encode(state: &Snapshot) -> serde_json::Value {
    let sources = state
        .source_index
        .iter()
        .map(|(root, paths)| (root, ordered(paths)))
        .collect::<BTreeMap<_, _>>();
    let artifacts = state
        .artifacts
        .iter()
        .map(|(key, topology)| {
            (
                key,
                (
                    ordered(&topology.files),
                    ordered(&topology.manifests),
                    ordered(&topology.aliases),
                    ordered(&topology.install_roots),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    serde_json::json!({
        "routes": text(&state.routes), "files": text(&state.files),
        "originals": text(ordered(&state.originals)), "shadows": text(ordered(&state.shadows)),
        "manifests": text(ordered(&state.manifests)), "fileOwners": text(ordered(&state.file_owners)),
        "manifestOwners": text(ordered(&state.manifest_owners)), "sourceIndex": text(sources),
        "sourceOwners": text(ordered(&state.source_owners)), "sourcePaths": text(ordered(&state.source_paths)),
        "rootOwners": text(sets(&state.root_owners)), "importerOwners": text(sets(&state.importer_owners)),
        "manifestRoutes": text(sets(&state.manifest_routes)), "shadowSources": text(sets(&state.shadow_sources)),
        "shadowScopes": text(ordered(&state.shadow_scopes)), "artifacts": text(artifacts),
        "reverse": text(sets(&state.reverse)), "aliases": text(ordered(&state.aliases)),
    })
}

fn work_value() -> serde_json::Value {
    let work = work();
    serde_json::json!({"calls": work.calls, "coldCalls": work.cold_calls,
        "pushed": work.pushed, "popped": work.popped, "maxPending": work.max_pending})
}

#[test]
fn complete_prepared_graph_observations() {
    let capture = std::env::var_os("VIZE_PACKAGE_ROUTE_GRAPH_CAPTURE").map(PathBuf::from);
    let graph = if capture.is_some() {
        let root = PathBuf::from(std::env::var_os("VIZE_PACKAGE_ROUTE_GRAPH_ROOT").unwrap());
        assert!(root.is_absolute());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        assert_eq!(std::fs::canonicalize(&root).unwrap(), root);
        Graph::at(root, None)
    } else {
        Graph::new()
    };
    let mut project = graph.registered();
    let registration = work_value();
    let mut states = Vec::new();
    history(&graph, &mut project, |name, project| {
        states.push(serde_json::json!({"state": name, "snapshot": encode(&snapshot(project, &graph)), "work": work_value()}));
    });
    let labels = states
        .iter()
        .map(|state| state["state"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        [
            "cold",
            "warm-noop",
            "warm-rebind",
            "owner-removed",
            "owner-repair",
            "all-removed",
            "all-repair"
        ]
    );
    if let Some(capture) = capture {
        std::fs::create_dir_all(&capture).unwrap();
        for state in &states {
            let output =
                capture.join(cstr!("{}.state.json", state["state"].as_str().unwrap()).as_str());
            assert!(!output.exists());
            std::fs::write(
                output,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "schema": "vize.controlled-package-shadow-state", "version": 1,
                    "fixtureRoot": graph.root, "virtualRoot": project.virtual_root,
                    "state": state["state"], "snapshot": state["snapshot"],
                    "nativeCheckerExecuted": false, "reporterProjectReproduced": false,
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let output = capture.join("work.json");
        assert!(!output.exists());
        std::fs::write(
            output,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": "vize.controlled-package-shadow-work", "version": 1,
                "registrationWork": registration,
                "states": states.iter().map(|state| serde_json::json!({"state": state["state"], "work": state["work"]})).collect::<Vec<_>>(),
                "nativeCheckerExecuted": false, "reporterProjectReproduced": false,
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
