//! Benchmarks for the package-route path collectors.
//!
//! Run with: cargo bench -p vize_canon --bench package_route
//!
//! The shape under test is a `workspace:*` dependency chain, which nests one
//! route per package. `collect_transitive_local_imports_with_resolver` calls
//! these collectors once per import occurrence, so their cost in the nesting
//! depth is what a large pnpm workspace pays (#4426).
//! The route-binding benchmark additionally measures the public batch-checker
//! caller that builds the production route/source indexes for the same shape.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::{fs, hint::black_box, path::Path};
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, PackageResolutionContext, PackageResolutionMode,
    PackageRoute, PackageRouteBinding,
};

const SOURCES_PER_PACKAGE: usize = 8;

/// One route per package, each nesting the next, as `workspace:*` produces.
fn chain(depth: usize, sources_per_package: usize) -> PackageRoute {
    chain_under(Path::new("/workspace"), depth, sources_per_package)
}

fn chain_under(workspace: &Path, depth: usize, sources_per_package: usize) -> PackageRoute {
    let mut route: Option<PackageRoute> = None;
    for level in (0..depth).rev() {
        let root = workspace.join(format!(
            "node_modules/.pnpm/@ws+lib{level:03}@1.0.0/node_modules/@ws/lib{level:03}"
        ));
        let source_paths = (0..sources_per_package)
            .map(|index| root.join(format!("src/c{index}.ts")))
            .collect::<Vec<_>>();
        let dependency_paths = (0..sources_per_package)
            .map(|index| root.join(format!("src/d{index}.ts")))
            .collect::<Vec<_>>();
        route = Some(PackageRoute {
            source_paths,
            dependency_paths,
            source_targets: Vec::new(),
            package_root: root.clone(),
            package_link_root: root.clone(),
            manifest_path: root.join("package.json"),
            package_name: Some(vize_l0::String::from(format!("@ws/lib{level:03}").as_str())),
            workspace_source: true,
            nested_routes: route.into_iter().collect(),
        });
    }
    route.expect("a chain has at least one route")
}

fn route_count(route: &PackageRoute) -> usize {
    1 + route.nested_routes.iter().map(route_count).sum::<usize>()
}

fn materialize_route_fixture(route: &PackageRoute) {
    fs::create_dir_all(route.package_root.join("src")).expect("package source root should exist");
    fs::write(
        &route.manifest_path,
        format!(
            "{{\"name\":\"{}\",\"type\":\"module\",\"exports\":\"./src/c0.ts\"}}\n",
            route
                .package_name
                .as_deref()
                .expect("workspace route should have a package name")
        ),
    )
    .expect("package manifest should write");
    for path in route.source_paths.iter().chain(&route.dependency_paths) {
        fs::write(path, "export const value = 1;\n").expect("package source should write");
    }
    for nested in &route.nested_routes {
        materialize_route_fixture(nested);
    }
}

fn route_binding_fixture(
    workspace: &Path,
    depth: usize,
    sources_per_package: usize,
) -> PackageRouteBinding {
    fs::create_dir_all(workspace.join("src")).expect("workspace source root should exist");
    fs::write(
        workspace.join("package.json"),
        "{\"name\":\"route-binding-bench\",\"private\":true,\"type\":\"module\"}\n",
    )
    .expect("workspace manifest should write");
    let importer_path = workspace.join("src/App.vue");
    fs::write(
        &importer_path,
        "<script setup lang=\"ts\">import '@ws/lib000';</script>\n",
    )
    .expect("workspace importer should write");

    let route = chain_under(workspace, depth, sources_per_package);
    materialize_route_fixture(&route);
    let invalidation_paths = route.invalidation_paths();
    PackageRouteBinding {
        importer_path,
        specifier: vize_l0::String::from("@ws/lib000"),
        occurrence_mode: PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths,
        shared_invalidation_paths: Vec::new(),
    }
}

fn assert_route_binding_fixture(
    binding: &PackageRouteBinding,
    workspace: &Path,
    depth: usize,
    sources_per_package: usize,
) {
    assert_eq!(binding.importer_path, workspace.join("src/App.vue"));
    assert!(binding.importer_path.is_file());
    assert_eq!(binding.specifier.as_str(), "@ws/lib000");
    let route = binding
        .route
        .as_ref()
        .expect("binding should retain its route");
    assert_eq!(route_count(route), depth);
    assert_eq!(
        route.all_source_paths().len(),
        depth * sources_per_package * 2
    );
    // Each package contributes its sources plus a distinct root and manifest;
    // manifest_path and package_link_root/package.json intentionally deduplicate.
    assert_eq!(
        binding.invalidation_paths.len(),
        depth * (sources_per_package * 2 + 2)
    );
    assert!(binding.invalidation_paths.iter().all(|path| path.exists()));

    let mut current = route;
    for level in 0..depth {
        assert!(current.workspace_source);
        assert!(current.manifest_path.is_file());
        assert_eq!(
            current.nested_routes.len(),
            if level + 1 < depth { 1 } else { 0 }
        );
        if let Some(nested) = current.nested_routes.first() {
            current = nested;
        }
    }
}

fn bench_collectors(criterion: &mut Criterion) {
    for depth in [16usize, 64, 130] {
        let route = chain(depth, SOURCES_PER_PACKAGE);
        criterion.bench_function(
            &format!("package_route/unambiguous_source_path/{depth}"),
            |b| b.iter(|| black_box(black_box(&route).unambiguous_source_path())),
        );
        criterion.bench_function(&format!("package_route/all_source_paths/{depth}"), |b| {
            b.iter(|| black_box(black_box(&route).all_source_paths().len()));
        });
        criterion.bench_function(&format!("package_route/invalidation_paths/{depth}"), |b| {
            b.iter(|| black_box(black_box(&route).invalidation_paths().len()));
        });
    }
}

fn bench_route_binding_index(criterion: &mut Criterion) {
    let corsa_path = std::env::current_exe().expect("benchmark executable should resolve");
    assert!(corsa_path.is_file());

    for depth in [16usize, 64, 130] {
        let fixture = tempfile::tempdir().expect("route binding fixture should create");
        let workspace = fixture
            .path()
            .canonicalize()
            .expect("route binding fixture should canonicalize");
        let binding = route_binding_fixture(&workspace, depth, SOURCES_PER_PACKAGE);
        assert_route_binding_fixture(&binding, &workspace, depth, SOURCES_PER_PACKAGE);

        criterion.bench_function(
            &format!("package_route/route_binding_index/{depth}"),
            |bencher| {
                bencher.iter_batched_ref(
                    || {
                        let checker = BatchTypeChecker::with_options_and_corsa_path(
                            &workspace,
                            BatchTypeCheckerOptions::default(),
                            Some(&corsa_path),
                        )
                        .expect("route binding checker should start");
                        (checker, Some(binding.clone()))
                    },
                    |(checker, binding)| {
                        checker.set_package_routes([binding
                            .take()
                            .expect("each benchmark input should have one binding")]);
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }
}

criterion_group!(benches, bench_collectors, bench_route_binding_index);
criterion_main!(benches);
