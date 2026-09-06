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
        cache.registration_entries(),
        1,
        "one package closure keeps one memo entry, however many importers reach it"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The bounded reachability scan reads and parses a package's dependency
/// closure, so repeating it for every authored importer of that package
/// reintroduces exactly the per-importer cost the registration memo removes
/// (#4137). Distinct importers keep distinct memo entries, so the scan itself
/// must be memoized against the route it inspects.
#[test]
fn package_route_reachability_is_scanned_once_per_route() {
    let root = tempfile::tempdir().unwrap();
    let first = write(
        root.path(),
        "src/first.ts",
        "import { widget } from 'widgets';\nvoid widget;\n",
    );
    let second = write(
        root.path(),
        "src/second.ts",
        "import { widget } from 'widgets';\nvoid widget;\n",
    );
    write(
        root.path(),
        "node_modules/widgets/package.json",
        r#"{"name":"widgets","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/widgets/index.d.ts",
        "export declare const widget: number;\n",
    );

    let mut canonical_paths = CanonicalPathCache::default();
    let first = canonical_paths.canonicalize(&first);
    let second = canonical_paths.canonicalize(&second);
    let mut packages = PackageRouteResolver::default();
    let mut cache = registration::VirtualRegistrationCache::default();

    for entry in [&first, &second] {
        let mut discovery = registration::VirtualRegistrationDiscovery::default();
        registration::non_relative_import_needs_virtual_registration(
            entry,
            &mut canonical_paths,
            ImportFileOptions::from(false),
            None,
            Some(&mut packages),
            &mut cache,
            &mut discovery,
        );
    }

    assert_eq!(
        cache.registration_entries(),
        2,
        "each authored source keeps its own memo entry"
    );
    assert_eq!(
        packages.metrics().reachability_checks,
        1,
        "one package route is scanned once, however many importers reach it"
    );
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

    assert!(
        !needs_registration,
        "an exhausted native route must not widen the program roots"
    );
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

#[test]
fn vue_runtime_support_stays_native_without_reachability_work() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(
        root.path(),
        "src/entry.ts",
        "import type { Component } from 'vue';\nexport type Entry = Component;\n",
    );
    write(
        root.path(),
        "node_modules/vue/package.json",
        r#"{"name":"vue","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/vue/index.d.ts",
        "export type { Component } from './runtime';\n",
    );
    write(
        root.path(),
        "node_modules/vue/runtime.d.ts",
        "export interface Component { readonly name?: string }\n",
    );

    let mut resolver = PackageRouteResolver::default();
    let discovered = collect_transitive_local_imports_with_resolver(
        &[entry],
        root.path(),
        &mut CanonicalPathCache::default(),
        false,
        None,
        &mut resolver,
    );

    assert!(discovered.registrations.is_empty());
    assert!(discovered.authored.is_empty());
    assert!(discovered.package_routes.is_empty());
    let metrics = resolver.metrics();
    assert_eq!(metrics.cache_misses, 0, "runtime support stays native");
    assert_eq!(
        metrics.reachability_checks, 0,
        "runtime support is terminal"
    );
}

#[test]
fn vue_dist_javascript_subpath_stays_native_when_allow_js() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(
        root.path(),
        "src/entry.ts",
        "import Vue from 'vue/dist/vue.cjs.js';\nexport const app = Vue;\n",
    );
    write(
        root.path(),
        "node_modules/vue/package.json",
        r#"{"name":"vue","main":"./index.js"}"#,
    );
    write(
        root.path(),
        "node_modules/vue/index.js",
        "module.exports = {}\n",
    );
    write(
        root.path(),
        "node_modules/vue/dist/vue.cjs.js",
        "module.exports = {}\n",
    );
    write(
        root.path(),
        "node_modules/vue/dist/vue.cjs.prod.js",
        "module.exports = {}\n",
    );

    let mut resolver = PackageRouteResolver::default();
    let discovered = collect_transitive_local_imports_with_resolver(
        &[entry],
        root.path(),
        &mut CanonicalPathCache::default(),
        ImportFileOptions {
            include_js: true,
            include_jsx: false,
        },
        None,
        &mut resolver,
    );

    assert!(discovered.registrations.is_empty());
    assert!(discovered.authored.is_empty());
    assert!(discovered.package_routes.is_empty());
    assert_eq!(resolver.metrics().cache_misses, 0);
}
