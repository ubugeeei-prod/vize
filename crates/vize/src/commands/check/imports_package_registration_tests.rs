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
