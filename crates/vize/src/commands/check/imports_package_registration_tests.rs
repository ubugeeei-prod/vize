use super::*;

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

/// Every SFC in a project imports the same handful of packages, so a package's
/// declaration closure must be walked once per package source, not once per
/// importer. Without the memo a 1,000-file check re-read and re-scanned Vue's
/// whole declaration graph 1,000 times (#4137). Deleting the closure after the
/// first call proves later callers replay the memo instead of walking again.
#[test]
fn package_aware_registration_walks_a_package_closure_once_per_source() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-package-memo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    write(
        &root,
        "App.vue",
        "<script setup lang=\"ts\"></script>\n<template><div /></template>\n",
    );
    write(
        &root,
        "node_modules/widgets/package.json",
        "{\"name\":\"widgets\",\"types\":\"index.d.ts\"}",
    );
    let entry = write(
        &root,
        "node_modules/widgets/index.d.ts",
        "import './internal';\nexport declare const widget: number;\n",
    );
    write(
        &root,
        "node_modules/widgets/internal.d.ts",
        "import '../../App.vue';\nexport declare const internal: number;\n",
    );

    let mut canonical_paths = CanonicalPathCache::default();
    let entry = canonical_paths.canonicalize(&entry);
    let mut packages = PackageRouteResolver::default();
    let mut cache = registration::VirtualRegistrationCache::default();

    let mut answers = Vec::new();
    for call in 0..8 {
        if call == 1 {
            // A fresh walk would now answer differently, so any later change of
            // answer means the closure was walked again.
            let _ = std::fs::remove_dir_all(root.join("node_modules/widgets"));
        }
        let mut discovery = registration::VirtualRegistrationDiscovery::default();
        let needs_registration = registration::non_relative_import_needs_virtual_registration(
            &entry,
            &mut canonical_paths,
            ImportFileOptions::from(false),
            None,
            Some(&mut packages),
            &mut cache,
            &mut discovery,
        );
        answers.push((
            needs_registration,
            discovery.package_routes.len(),
            discovery.package_sources.len(),
        ));
    }

    assert!(answers[0].0, "the closure reaches an SFC: {:?}", answers[0]);
    assert!(
        answers.windows(2).all(|pair| pair[0] == pair[1]),
        "memoized answers must replay the walked answer: {answers:?}"
    );
    assert_eq!(
        cache.len(),
        1,
        "one package closure keeps one memo entry, however many importers reach it"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn exhausted_package_closure_tracks_inputs_without_materializing_a_shadow() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(
        root.path(),
        "src/entry.ts",
        "import { Chart } from 'chart-like';\nvoid Chart;\n",
    );
    write(
        root.path(),
        "node_modules/chart-like/package.json",
        r#"{"name":"chart-like","exports":{".":{"types":"./index.d.ts","import":"./index.js"}}}"#,
    );
    write(
        root.path(),
        "node_modules/chart-like/index.d.ts",
        "export declare class Chart {}\n",
    );
    write(
        root.path(),
        "node_modules/chart-like/index.js",
        &"x".repeat(129 * 1024),
    );
    let mut canonical_paths = CanonicalPathCache::default();
    let entry = canonical_paths.canonicalize(&entry);
    let mut packages = PackageRouteResolver::default();
    let mut cache = registration::VirtualRegistrationCache::default();
    let mut discovery = registration::VirtualRegistrationDiscovery::default();

    let needs_registration = registration::non_relative_import_needs_virtual_registration(
        &entry,
        &mut canonical_paths,
        ImportFileOptions {
            include_js: true,
            include_jsx: true,
        },
        None,
        Some(&mut packages),
        &mut cache,
        &mut discovery,
    );

    assert!(needs_registration);
    assert_eq!(discovery.package_routes.len(), 1);
    let binding = &discovery.package_routes[0];
    assert!(binding.route.is_none(), "an exhausted route stays native");
    assert!(
        binding
            .invalidation_paths
            .iter()
            .any(|path| { path.ends_with("node_modules/chart-like/package.json") })
    );
    assert!(
        binding
            .invalidation_paths
            .iter()
            .any(|path| path.ends_with("node_modules/chart-like/index.js"))
    );
    let metrics = packages.metrics();
    assert_eq!(metrics.reachability_checks, 1);
    assert_eq!(metrics.reachability_budget_exceeded, 1);
    assert_eq!(metrics.last_reachability_files, 2);
    assert_eq!(metrics.last_reachability_parses, 1);
}
