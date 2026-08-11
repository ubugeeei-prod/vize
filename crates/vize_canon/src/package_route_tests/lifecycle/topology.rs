#![allow(clippy::disallowed_methods)]

use super::super::super::{
    PackageResolutionContext, PackageResolutionMode, PackageRouteResolver, PackageSourceOptions,
};

#[test]
fn builtins_and_existing_javascript_only_packages_are_not_cold_create_watches() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    let package = root.path().join("app/node_modules/runtime-only");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"runtime-only","main":"./index.js"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.js"), "exports.value = 1\n").unwrap();
    let mut resolver = PackageRouteResolver::default();

    for specifier in ["node:fs", "runtime-only"] {
        let lookup = resolver.lookup(&importer, specifier, PackageSourceOptions::default());
        assert!(lookup.clone().into_parts().0.is_none());
        assert!(
            !lookup.is_watchable_negative(),
            "{specifier} must not add a persistent package-create binding"
        );
    }
}

#[test]
fn one_resolver_recovers_when_an_authored_runtime_twin_is_created() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    let package = root.path().join("app");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{".":"./dist/index.js"}}"#,
    )
    .unwrap();
    let mut resolver = PackageRouteResolver::default();
    let negative = resolver.lookup(&importer, "@scope/ui", PackageSourceOptions::default());
    assert!(negative.is_watchable_negative());
    let (_, inputs) = negative.into_parts();
    let widget = package.join("dist/index.vue");
    assert!(inputs.contains(&package.canonicalize().unwrap().join("dist/index.vue")));
    std::fs::write(&widget, "<template />\n").unwrap();
    assert_eq!(
        resolver
            .resolve(&importer, "@scope/ui", PackageSourceOptions::default())
            .unwrap()
            .unambiguous_source_path(),
        Some(&widget.canonicalize().unwrap())
    );
}

#[test]
fn one_resolver_detects_a_same_mtime_same_length_manifest_retarget() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    let package = root.path().join("app/node_modules/ui");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(package.join("src")).unwrap();
    let manifest = package.join("package.json");
    let first_manifest = r#"{"exports":{".":"./src/Alpha.vue"}}"#;
    let second_manifest = r#"{"exports":{".":"./src/Bravo.vue"}}"#;
    assert_eq!(first_manifest.len(), second_manifest.len());
    std::fs::write(&manifest, first_manifest).unwrap();
    let alpha = package.join("src/Alpha.vue");
    let bravo = package.join("src/Bravo.vue");
    std::fs::write(&alpha, "<template />\n").unwrap();
    std::fs::write(&bravo, "<template />\n").unwrap();
    let mut resolver = PackageRouteResolver::default();
    assert_eq!(
        resolver
            .resolve(&importer, "ui", PackageSourceOptions::default())
            .unwrap()
            .unambiguous_source_path(),
        Some(&alpha.canonicalize().unwrap())
    );
    let modified = std::fs::metadata(&manifest).unwrap().modified().unwrap();
    std::fs::write(&manifest, second_manifest).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&manifest)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    assert_eq!(
        resolver
            .resolve(&importer, "ui", PackageSourceOptions::default())
            .unwrap()
            .unambiguous_source_path(),
        Some(&bravo.canonicalize().unwrap())
    );
}

#[test]
#[cfg(unix)]
fn symlinked_workspace_route_records_link_and_real_manifest_inputs() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let package = root.path().join("packages/ui");
    let link = app.join("node_modules/@scope/ui");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{".":"./src/Widget.vue"}}"#,
    )
    .unwrap();
    let widget = package.join("src/Widget.vue");
    std::fs::write(&widget, "<template />\n").unwrap();
    symlink(&package, &link).unwrap();
    let route = PackageRouteResolver::default()
        .resolve(
            &app.join("src"),
            "@scope/ui",
            PackageSourceOptions::default(),
        )
        .unwrap();
    let inputs = route.invalidation_paths();
    assert!(route.workspace_source);
    assert_eq!(
        route.unambiguous_source_path(),
        Some(&widget.canonicalize().unwrap())
    );
    assert_eq!(route.package_link_root, link);
    assert_eq!(route.package_root, package.canonicalize().unwrap());
    assert_ne!(route.package_link_root, route.package_root);
    assert_ne!(
        route.package_link_root.join("package.json"),
        route.manifest_path
    );
    assert!(inputs.contains(&route.package_link_root));
    assert!(inputs.contains(&route.package_link_root.join("package.json")));
    assert!(inputs.contains(&route.manifest_path));
    assert!(inputs.contains(&route.unambiguous_source_path().unwrap().clone()));
}

#[test]
#[cfg(unix)]
fn one_resolver_detects_a_workspace_package_symlink_retarget() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let first_package = root.path().join("packages/ui-a");
    let second_package = root.path().join("packages/ui-b");
    let link = app.join("node_modules/@scope/ui");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    for package in [&first_package, &second_package] {
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{"name":"@scope/ui","exports":"./src/Widget.vue"}"#,
        )
        .unwrap();
        std::fs::write(package.join("src/Widget.vue"), "<template />\n").unwrap();
    }
    symlink(&first_package, &link).unwrap();
    let mut resolver = PackageRouteResolver::default();
    assert_eq!(
        resolver
            .resolve(
                &app.join("src"),
                "@scope/ui",
                PackageSourceOptions::default(),
            )
            .unwrap()
            .package_root,
        first_package.canonicalize().unwrap()
    );
    std::fs::remove_file(&link).unwrap();
    symlink(&second_package, &link).unwrap();
    assert_eq!(
        resolver
            .resolve(
                &app.join("src"),
                "@scope/ui",
                PackageSourceOptions::default(),
            )
            .unwrap()
            .package_root,
        second_package.canonicalize().unwrap()
    );
}

#[test]
fn effective_resolution_context_partitions_route_cache_identity() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("src");
    let package = root.path().join("node_modules/ui");
    std::fs::create_dir_all(&importer).unwrap();
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":{"browser":"./src/Browser.vue","default":"./src/Default.vue"}}}"#,
    )
    .unwrap();
    for file in ["Browser.vue", "Default.vue"] {
        std::fs::write(package.join("src").join(file), "<template />\n").unwrap();
    }
    let mut resolver = PackageRouteResolver::default();
    for (mode, conditions) in [
        (PackageResolutionMode::Import, vec!["browser"]),
        (PackageResolutionMode::Require, vec!["node"]),
    ] {
        resolver.lookup_with_context(
            &importer,
            "ui",
            PackageSourceOptions::default(),
            PackageResolutionContext::new(Some("bundler"), mode, conditions),
        );
    }
    assert_eq!(resolver.metrics().cache_misses, 2);
    assert_eq!(resolver.metrics().cache_hits, 0);
}
