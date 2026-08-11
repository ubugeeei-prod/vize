#![allow(clippy::disallowed_methods)]

use super::super::{PackageRouteResolver, PackageSourceOptions};
use vize_carton::cstr;

#[path = "lifecycle/topology.rs"]
mod topology;

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
    assert!(lookup.is_watchable_negative());
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
            .unambiguous_source_path()
            .unwrap()
            .clone(),
        widget.canonicalize().unwrap()
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
    assert!(
        first
            .unambiguous_source_path()
            .unwrap()
            .clone()
            .ends_with("First.vue")
    );
    assert!(
        second
            .unambiguous_source_path()
            .unwrap()
            .clone()
            .ends_with("Other.vue")
    );
}

#[test]
fn warm_cache_and_source_invalidation_are_scoped_to_the_affected_route() {
    let root = tempfile::tempdir().unwrap();
    let mut resolver = PackageRouteResolver::default();
    let mut importers = Vec::new();
    let mut sources = Vec::new();
    for app in ["alpha", "bravo"] {
        let importer = root.path().join(app).join("src");
        let package = root.path().join(app).join("node_modules/ui");
        std::fs::create_dir_all(&importer).unwrap();
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{"exports":"./src/Root.vue"}"#,
        )
        .unwrap();
        let source = package.join("src/Root.vue");
        std::fs::write(&source, "<template>A</template>\n").unwrap();
        importers.push(importer);
        sources.push(source);
    }

    for importer in &importers {
        resolver
            .resolve(importer, "ui", PackageSourceOptions::default())
            .unwrap();
    }
    for importer in &importers {
        resolver
            .resolve(importer, "ui", PackageSourceOptions::default())
            .unwrap();
    }
    assert_eq!(resolver.metrics().cache_misses, 2);
    assert_eq!(resolver.metrics().cache_hits, 2);

    let modified = std::fs::metadata(&sources[0]).unwrap().modified().unwrap();
    std::fs::write(&sources[0], "<template>B</template>\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&sources[0])
        .unwrap()
        .set_modified(modified)
        .unwrap();
    for importer in &importers {
        resolver
            .resolve(importer, "ui", PackageSourceOptions::default())
            .unwrap();
    }
    assert_eq!(resolver.metrics().cache_misses, 3);
    assert_eq!(resolver.metrics().cache_hits, 3);

    // A lockfile is an invalidation trigger for both installed topologies; it
    // is never parsed as the route authority.
    std::fs::write(
        root.path().join("pnpm-lock.yaml"),
        "lockfileVersion: '9.0'\n",
    )
    .unwrap();
    for importer in &importers {
        resolver
            .resolve(importer, "ui", PackageSourceOptions::default())
            .unwrap();
    }
    assert_eq!(resolver.metrics().cache_misses, 5);
    assert_eq!(resolver.metrics().cache_hits, 3);
}
