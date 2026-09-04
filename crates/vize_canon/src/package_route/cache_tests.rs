use super::{PackageRouteResolver, RESOLUTION_CACHE_CAPACITY};

#[test]
fn route_cache_has_a_measured_hard_bound() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("src");
    std::fs::create_dir_all(&importer).unwrap();
    let mut resolver = PackageRouteResolver::default();
    for index in 0..=RESOLUTION_CACHE_CAPACITY {
        let _ = resolver.lookup(
            &importer,
            &format!("package-{index}"),
            crate::PackageSourceOptions::new(false, false),
        );
    }

    let metrics = resolver.metrics();
    assert_eq!(
        metrics.resolution_cache_entries,
        RESOLUTION_CACHE_CAPACITY as u64
    );
    assert_eq!(metrics.resolution_cache_evictions, 1);
}

#[test]
fn validation_epoch_reuses_stamp_snapshots_for_shared_route_inputs() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("packages/app/src");
    std::fs::create_dir_all(&importer).unwrap();
    write_package(
        root.path(),
        "pkg-a",
        r#"{"name":"pkg-a","types":"./index.ts"}"#,
    );
    write_package(
        root.path(),
        "pkg-b",
        r#"{"name":"pkg-b","types":"./index.ts"}"#,
    );
    let mut resolver = PackageRouteResolver::default();
    resolver.begin_validation_epoch();

    let _ = resolver.lookup(
        &importer,
        "pkg-a",
        crate::PackageSourceOptions::new(false, false),
    );
    let _ = resolver.lookup(
        &importer,
        "pkg-b",
        crate::PackageSourceOptions::new(false, false),
    );

    let (canonical_paths, stamp_paths, stamp_captures) = resolver.debug_validation_cache_counts();
    assert!(canonical_paths > 0);
    assert!(stamp_paths > 0);
    assert_eq!(stamp_captures, stamp_paths);

    let _ = resolver.lookup(
        &importer,
        "pkg-a",
        crate::PackageSourceOptions::new(false, false),
    );
    let (_, _, repeated_stamp_captures) = resolver.debug_validation_cache_counts();
    assert_eq!(repeated_stamp_captures, stamp_captures);
}

fn write_package(root: &std::path::Path, name: &str, manifest: &str) {
    let package = root.join("packages/app/node_modules").join(name);
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("package.json"), manifest).unwrap();
    std::fs::write(package.join("index.ts"), "export const value = 1;\n").unwrap();
}
