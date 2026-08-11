use vize_carton::FxHashMap;

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
use crate::corsa_bridge::vue_dependencies_alias::context::routes::RouteDiscovery;

#[test]
fn exhausted_editor_route_keeps_native_spelling_and_invalidation_inputs() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("src/App.vue");
    let package = root.path().join("node_modules/chart-like");
    let manifest = package.join("package.json");
    let oversized = package.join("dist/index.js");
    write(&root.path().join("tsconfig.json"), "{}\n");
    write(
        &host,
        "<script setup lang=\"ts\">import { Chart } from 'chart-like'; void Chart</script>\n",
    );
    write(
        &manifest,
        r#"{"name":"chart-like","exports":{".":{"types":"./dist/index.d.ts","import":"./dist/index.js"}}}"#,
    );
    write(
        &package.join("dist/index.d.ts"),
        "export declare class Chart {}\n",
    );
    write(&oversized, &"x".repeat(129 * 1024));

    let source = std::fs::read_to_string(&host).unwrap();
    let context = AliasContext::for_host(&host, &source, &FxHashMap::default());

    assert!(context.aliases.is_empty());
    assert!(context.package_routes.is_empty());
    assert!(context.mirror.is_none());
    assert!(
        context
            .route_inputs
            .iter()
            .any(|path| { path.ends_with("node_modules/chart-like/package.json") })
    );
    assert!(
        context
            .route_inputs
            .iter()
            .any(|path| path.ends_with("node_modules/chart-like/dist/index.js"))
    );

    // The assertions above also hold for a completed scan, so pin the outcome
    // itself: removing the byte guard turns this into a finished scan and drops
    // the exhaustion counter back to zero.
    let settings =
        crate::batch::virtual_project::package_resolution::PackageResolutionSettings::default();
    let mut resolver = crate::PackageRouteResolver::default();
    let mut routes = FxHashMap::default();
    let mut reachability = FxHashMap::default();
    let mut bindings = Vec::new();
    let mut inputs = Vec::new();
    let mut discovery = RouteDiscovery::new(
        &settings,
        &mut resolver,
        &mut routes,
        &mut reachability,
        &mut bindings,
        &mut inputs,
        &[],
    );
    assert!(!discovery.resolve(&host, "chart-like", crate::PackageResolutionMode::Import));
    drop(discovery);

    let metrics = resolver.metrics();
    assert_eq!(metrics.reachability_checks, 1);
    assert_eq!(metrics.reachability_budget_exceeded, 1);
}

#[test]
fn reachability_cache_separates_importer_and_resolution_context() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("src/entry.ts");
    let other_importer = root.path().join("other/entry.ts");
    let package = root.path().join("node_modules/typed-package");
    write(&importer, "import type { Value } from 'typed-package';\n");
    write(
        &other_importer,
        "import type { Value } from 'typed-package';\n",
    );
    write(
        &package.join("package.json"),
        r#"{"name":"typed-package","types":"./index.d.ts"}"#,
    );
    write(&package.join("index.d.ts"), "export interface Value {}\n");
    let settings =
        crate::batch::virtual_project::package_resolution::PackageResolutionSettings::default();
    let mut resolver = crate::PackageRouteResolver::default();
    let mut routes = FxHashMap::default();
    let mut reachability = FxHashMap::default();
    let mut bindings = Vec::new();
    let mut inputs = Vec::new();
    let mut discovery = RouteDiscovery::new(
        &settings,
        &mut resolver,
        &mut routes,
        &mut reachability,
        &mut bindings,
        &mut inputs,
        &[],
    );

    assert!(!discovery.resolve(
        &importer,
        "typed-package",
        crate::PackageResolutionMode::Import,
    ));
    assert!(!discovery.resolve(
        &importer,
        "typed-package",
        crate::PackageResolutionMode::Require,
    ));
    assert!(!discovery.resolve(
        &other_importer,
        "typed-package",
        crate::PackageResolutionMode::Import,
    ));
    // A repeated resolution is a cache hit, so it must not scan again or move
    // the cumulative work counters.
    assert!(!discovery.resolve(
        &importer,
        "typed-package",
        crate::PackageResolutionMode::Import,
    ));
    drop(discovery);

    assert_eq!(reachability.len(), 3);
    assert_eq!(resolver.metrics().reachability_checks, 3);
}

fn write(path: &std::path::Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
