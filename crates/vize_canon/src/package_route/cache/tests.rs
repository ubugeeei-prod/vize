#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
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

#[test]
fn validation_scope_validates_once_and_revalidates_after_it_ends() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("packages/app/src");
    std::fs::create_dir_all(&importer).unwrap();
    write_package(
        root.path(),
        "pkg-a",
        r#"{"name":"pkg-a","types":"./index.ts"}"#,
    );
    let options = crate::PackageSourceOptions::new(false, false);
    let mut resolver = PackageRouteResolver::default();
    let types =
        |lookup: super::PackageRouteLookup| lookup.into_parts().0.map(|route| route.source_paths);
    let before = types(resolver.lookup(&importer, "pkg-a", options));
    assert!(before.is_some());

    {
        let _scope = resolver.validation_scope();
        let _ = resolver.lookup(&importer, "pkg-a", options);
        let (_, _, captures) = resolver.debug_validation_cache_counts();
        let _nested = resolver.validation_scope();
        let _ = resolver.lookup(&importer, "pkg-a", options);
        let _ = resolver.lookup(&importer, "pkg-a", options);
        assert_eq!(resolver.debug_validation_cache_counts().2, captures);
    }

    // Outside the scope a manifest edit must be seen again.
    std::fs::write(
        root.path()
            .join("packages/app/node_modules/pkg-a/package.json"),
        r#"{"name":"pkg-a","types":"./other.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        root.path()
            .join("packages/app/node_modules/pkg-a/other.d.ts"),
        "export declare const value: 1;\n",
    )
    .unwrap();
    let after = types(resolver.lookup(&importer, "pkg-a", options));
    assert_ne!(after, before);

    // A later scope gets a fresh epoch, so it revalidates too.
    std::fs::write(
        root.path()
            .join("packages/app/node_modules/pkg-a/package.json"),
        r#"{"name":"pkg-a","types":"./index.ts"}"#,
    )
    .unwrap();
    let _scope = resolver.validation_scope();
    assert_eq!(types(resolver.lookup(&importer, "pkg-a", options)), before);
}

#[test]
fn concurrent_misses_reuse_manifests_and_revalidate_in_a_new_epoch() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("packages/app/src");
    let nested = importer.join("nested");
    let deeper = nested.join("deeper");
    std::fs::create_dir_all(&deeper).unwrap();
    write_package(
        root.path(),
        "pkg-a",
        r#"{"name":"pkg-a","types":"./index.ts"}"#,
    );
    let options = crate::PackageSourceOptions::new(false, false);
    let context = crate::PackageResolutionContext::default();
    let types =
        |lookup: super::PackageRouteLookup| lookup.into_parts().0.map(|route| route.source_paths);
    let mut resolver = PackageRouteResolver::default();
    resolver.begin_validation_epoch();

    let before = types(resolver.lookup_with_context_concurrent(
        &importer,
        "pkg-a",
        options,
        context.clone(),
    ));
    assert!(before.is_some());
    let reads = resolver.metrics().manifest_reads;
    assert!(reads > 0);
    // Another importer directory is a resolution miss. Only that directory's
    // own manifest is new; the package and ancestor manifests are reused.
    let _ = resolver.lookup_with_context_concurrent(&nested, "pkg-a", options, context.clone());
    assert_eq!(resolver.metrics().manifest_reads, reads + 1);

    std::fs::write(
        root.path()
            .join("packages/app/node_modules/pkg-a/package.json"),
        r#"{"name":"pkg-a","types":"./other.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        root.path()
            .join("packages/app/node_modules/pkg-a/other.d.ts"),
        "export declare const value: 1;\n",
    )
    .unwrap();
    resolver.begin_validation_epoch();
    let after = types(resolver.lookup_with_context_concurrent(&deeper, "pkg-a", options, context));
    assert!(after.is_some());
    assert_ne!(after, before);
}

fn write_package(root: &std::path::Path, name: &str, manifest: &str) {
    let package = root.join("packages/app/node_modules").join(name);
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("package.json"), manifest).unwrap();
    std::fs::write(package.join("index.ts"), "export const value = 1;\n").unwrap();
}
