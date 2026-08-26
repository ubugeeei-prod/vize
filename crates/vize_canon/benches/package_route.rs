//! Benchmarks for the package-route path collectors.
//!
//! Run with: cargo bench -p vize_canon --bench package_route
//!
//! The shape under test is a `workspace:*` dependency chain, which nests one
//! route per package. `collect_transitive_local_imports_with_resolver` calls
//! these collectors once per import occurrence, so their cost in the nesting
//! depth is what a large pnpm workspace pays (#4426).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::{hint::black_box, path::PathBuf};
use vize_canon::PackageRoute;

/// One route per package, each nesting the next, as `workspace:*` produces.
fn chain(depth: usize, sources_per_package: usize) -> PackageRoute {
    let mut route: Option<PackageRoute> = None;
    for level in (0..depth).rev() {
        let root = PathBuf::from(format!(
            "/workspace/node_modules/.pnpm/@ws+lib{level:03}@1.0.0/node_modules/@ws/lib{level:03}"
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
            package_name: Some(vize_s0::String::from(format!("@ws/lib{level:03}").as_str())),
            workspace_source: true,
            nested_routes: route.into_iter().collect(),
        });
    }
    route.expect("a chain has at least one route")
}

fn bench_collectors(criterion: &mut Criterion) {
    for depth in [16usize, 64, 130] {
        let route = chain(depth, 8);
        criterion.bench_function(&format!("package_route/all_source_paths/{depth}"), |b| {
            b.iter(|| black_box(black_box(&route).all_source_paths().len()));
        });
        criterion.bench_function(&format!("package_route/invalidation_paths/{depth}"), |b| {
            b.iter(|| black_box(black_box(&route).invalidation_paths().len()));
        });
    }
}

criterion_group!(benches, bench_collectors);
criterion_main!(benches);
