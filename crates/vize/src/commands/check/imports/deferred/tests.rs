use super::super::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, String,
    sort_package_route_bindings,
};
use super::*;
use vize_canon::PackageRouteSource;
use vize_l0::cstr;

fn paths(values: &[&str]) -> Vec<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

fn pending(label: &str, sources: &[&str], package: &str) -> DeferredWorkspaceRoute {
    let package_root = PathBuf::from(package);
    DeferredWorkspaceRoute {
        importer_path: PathBuf::from(cstr!("/app/{label}.vue").as_str()),
        specifier: String::from("@workspace/pkg"),
        occurrence_mode: PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: PackageRoute {
            source_paths: paths(sources),
            dependency_paths: Vec::new(),
            source_targets: Vec::new(),
            manifest_path: package_root.join("package.json"),
            package_link_root: PathBuf::from(cstr!("/links/{label}").as_str()),
            package_root,
            package_name: Some(String::from("@workspace/pkg")),
            workspace_source: true,
            nested_routes: Vec::new(),
        },
        invalidation_paths: paths(&["/config/tsconfig.json"]),
        package_graph: false,
    }
}

fn graph() -> ImportReachabilityGraph {
    let mut graph = ImportReachabilityGraph::default();
    for (from, to) in [
        ("/pkg/a.ts", "/pkg/b.ts"),
        ("/pkg/a.ts", "/pkg/c.ts"),
        ("/pkg/a.ts", "/outside/external.ts"),
        ("/pkg/b.ts", "/pkg/nested/Leaf.vue"),
        ("/pkg/c.ts", "/pkg/nested/Leaf.vue"),
        ("/pkg/nested/Leaf.vue", "/pkg/a.ts"),
        ("/pkg/requires/index.ts", "/pkg/requires/Other.vue"),
    ] {
        graph.add_edge(&PathBuf::from(from), &PathBuf::from(to));
    }
    graph
}

fn requests() -> Vec<DeferredWorkspaceRoute> {
    let mut first = pending("first", &["/pkg/a.ts"], "/pkg");
    first.route.dependency_paths = paths(&["/pkg/retained.ts"]);
    let duplicate = pending("duplicate", &["/pkg/a.ts", "/pkg/a.ts"], "/pkg");
    let boundary = pending("boundary", &["/pkg/a.ts"], "/pkg/nested");
    let mut conditional = pending("conditional", &["/pkg/requires/index.ts"], "/pkg");
    conditional.occurrence_mode = PackageResolutionMode::Require;
    conditional.context.mode = PackageResolutionMode::Require;
    conditional.context.active_conditions = vec![String::from("custom-condition")];
    conditional.package_graph = true;
    let negative = pending("negative", &["/pkg/plain.ts"], "/pkg");
    let mut forced = pending("forced", &["/forced/source.ts"], "/forced");
    forced.route.source_targets.push(PackageRouteSource {
        target_path: PathBuf::from("/forced/runtime.js"),
        source_path: PathBuf::from("/forced/source.ts"),
        native_probe_path: PathBuf::from("/forced/runtime.ts"),
    });
    vec![first, duplicate, boundary, conditional, negative, forced]
}

#[test]
fn repeated_routes_share_traversals_without_merging_conditions_or_package_boundaries() {
    let graph = graph();
    let mut registered = paths(&["/app/first.vue"]).into_iter().collect();
    let mut registrations = Vec::new();
    let mut bindings = Vec::new();
    let work = complete_routes(
        &graph,
        requests(),
        &mut registered,
        &mut registrations,
        &mut bindings,
    );
    assert_eq!(
        work,
        CompletionWork {
            requests: 6,
            eligible: 5,
            traversals: 3,
            reachable_nodes: 7
        }
    );
    assert_eq!(
        registrations,
        paths(&["/app/duplicate.vue", "/app/boundary.vue", "/app/forced.vue"])
    );
    assert_eq!(
        registered,
        paths(&[
            "/app/first.vue",
            "/app/duplicate.vue",
            "/app/boundary.vue",
            "/app/forced.vue"
        ])
        .into_iter()
        .collect::<FxHashSet<_>>()
    );
    let mut expected = Vec::new();
    for pending in requests() {
        let name = pending.importer_path.file_stem().unwrap().to_str().unwrap();
        let dependencies: &[&str] = match name {
            "first" => &[
                "/pkg/a.ts",
                "/pkg/b.ts",
                "/pkg/c.ts",
                "/pkg/nested/Leaf.vue",
                "/pkg/retained.ts",
            ],
            "duplicate" => &[
                "/pkg/a.ts",
                "/pkg/b.ts",
                "/pkg/c.ts",
                "/pkg/nested/Leaf.vue",
            ],
            "boundary" => &["/pkg/nested/Leaf.vue"],
            "conditional" => &["/pkg/requires/Other.vue", "/pkg/requires/index.ts"],
            "forced" => &[],
            "negative" => continue,
            _ => unreachable!(),
        };
        let mut route = pending.route;
        route.dependency_paths = paths(dependencies);
        expected.push(PackageRouteBinding {
            importer_path: pending.importer_path,
            specifier: pending.specifier,
            occurrence_mode: pending.occurrence_mode,
            context: pending.context,
            route: Some(route),
            invalidation_paths: pending.invalidation_paths,
            shared_invalidation_paths: Vec::new(),
        });
    }
    sort_package_route_bindings(&mut bindings);
    sort_package_route_bindings(&mut expected);
    assert_eq!(bindings, expected);
}

#[test]
fn a_new_collection_observes_changed_edges_and_empty_requests_do_no_work() {
    let mut graph = graph();
    let mut registered = FxHashSet::default();
    let mut registrations = Vec::new();
    let mut bindings = Vec::new();
    assert_eq!(
        complete_routes(
            &graph,
            Vec::new(),
            &mut registered,
            &mut registrations,
            &mut bindings
        ),
        CompletionWork::default()
    );
    complete_routes(
        &graph,
        vec![pending("before", &["/pkg/a.ts"], "/pkg")],
        &mut registered,
        &mut registrations,
        &mut bindings,
    );
    let first = bindings.pop().unwrap().route.unwrap();
    assert_eq!(
        first.dependency_paths,
        paths(&[
            "/pkg/a.ts",
            "/pkg/b.ts",
            "/pkg/c.ts",
            "/pkg/nested/Leaf.vue"
        ])
    );
    graph.edges.clear();
    graph.add_edge(&PathBuf::from("/pkg/a.ts"), &PathBuf::from("/pkg/New.vue"));
    complete_routes(
        &graph,
        vec![pending("after", &["/pkg/a.ts"], "/pkg")],
        &mut registered,
        &mut registrations,
        &mut bindings,
    );
    assert_eq!(
        bindings.pop().unwrap().route.unwrap().dependency_paths,
        paths(&["/pkg/New.vue", "/pkg/a.ts"])
    );
}
