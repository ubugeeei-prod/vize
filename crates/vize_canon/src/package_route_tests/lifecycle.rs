#![allow(clippy::disallowed_methods)]

use super::super::{PackageRouteResolver, PackageSourceOptions};
use vize_carton::cstr;

#[test]
fn unresolved_packages_retain_searched_link_and_manifest_candidates() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    std::fs::create_dir_all(&importer).unwrap();
    let link = root.path().join("app/node_modules/@scope/ui");
    let lookup = PackageRouteResolver::default().lookup(
        &importer,
        "@scope/ui/widget",
        PackageSourceOptions::default(),
    );
    let (route, inputs) = lookup.into_parts();

    assert!(route.is_none());
    assert!(inputs.contains(&link));
    assert!(inputs.contains(&link.join("package.json")));
}

#[test]
fn one_resolver_recovers_after_a_negative_package_route_is_created() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    let package = root.path().join("app/node_modules/@scope/ui");
    std::fs::create_dir_all(&importer).unwrap();
    let mut resolver = PackageRouteResolver::default();

    assert!(
        resolver
            .resolve(
                &importer,
                "@scope/ui/widget",
                PackageSourceOptions::default(),
            )
            .is_none()
    );

    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{"./widget":"./src/Widget.vue"}}"#,
    )
    .unwrap();
    let widget = package.join("src/Widget.vue");
    std::fs::write(&widget, "<template />\n").unwrap();

    assert_eq!(
        resolver
            .resolve(
                &importer,
                "@scope/ui/widget",
                PackageSourceOptions::default(),
            )
            .unwrap()
            .source_path,
        widget.canonicalize().unwrap()
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
            .source_path,
        alpha.canonicalize().unwrap()
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
            .source_path,
        bravo.canonicalize().unwrap()
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
    assert_eq!(route.source_path, widget.canonicalize().unwrap());
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
    assert!(inputs.contains(&route.source_path));
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
#[cfg(unix)]
fn cache_keeps_distinct_logical_importers_with_one_physical_identity() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let shared = root.path().join("shared/src");
    std::fs::create_dir_all(&shared).unwrap();
    let first_app = root.path().join("first-app");
    let second_app = root.path().join("second-app");
    std::fs::create_dir_all(&first_app).unwrap();
    std::fs::create_dir_all(&second_app).unwrap();
    symlink(&shared, first_app.join("src")).unwrap();
    symlink(&shared, second_app.join("src")).unwrap();
    for (app, target) in [(&first_app, "First.vue"), (&second_app, "Other.vue")] {
        let package = app.join("node_modules/ui");
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(
            package.join("package.json"),
            cstr!(r#"{{"exports":"./src/{target}"}}"#).as_str(),
        )
        .unwrap();
        std::fs::write(package.join("src").join(target), "<template />\n").unwrap();
    }
    assert_eq!(
        first_app.join("src").canonicalize().unwrap(),
        second_app.join("src").canonicalize().unwrap()
    );
    let mut resolver = PackageRouteResolver::default();

    let first = resolver
        .resolve(
            &first_app.join("src"),
            "ui",
            PackageSourceOptions::default(),
        )
        .unwrap();
    let second = resolver
        .resolve(
            &second_app.join("src"),
            "ui",
            PackageSourceOptions::default(),
        )
        .unwrap();
    assert!(first.source_path.ends_with("First.vue"));
    assert!(second.source_path.ends_with("Other.vue"));
}
