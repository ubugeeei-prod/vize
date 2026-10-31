use std::path::Path;
use std::sync::Arc;

use vize_canon::{PackageRouteResolver, PackageSourceOptions};

use super::CachedPackageLookup;

fn write_package(root: &Path) {
    let package = root.join("app/node_modules/ui");
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"ui","exports":{".":"./src/Widget.vue"}}"#,
    )
    .unwrap();
    std::fs::write(package.join("src/Widget.vue"), "<template />\n").unwrap();
}

#[test]
fn shared_lookup_preserves_complete_route_and_independent_binding_inputs() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    std::fs::create_dir_all(&importer).unwrap();
    write_package(root.path());
    let original =
        PackageRouteResolver::default().lookup(&importer, "ui", PackageSourceOptions::default());
    let expected = original.clone().into_parts();
    let cached = CachedPackageLookup::from(original);
    assert!(!cached.is_watchable_negative());
    let (route, mut inputs) = cached.clone().into_parts();
    let route = route.unwrap();
    assert_eq!(Some(route.as_ref()), expected.0.as_ref());
    assert_eq!(inputs, expected.1);
    assert!(Arc::ptr_eq(&route, cached.route.as_ref().unwrap()));

    // A published binding owns additions, while later importers still see the
    // exact resolver answer and input order rather than the first binding.
    let mut binding_route = route.as_ref().clone();
    binding_route
        .dependency_paths
        .push(importer.join("owned.ts"));
    inputs.push(importer.join("context.json"));
    let (later_route, later_inputs) = cached.into_parts();
    assert_eq!(later_route.as_deref(), expected.0.as_ref());
    assert_eq!(later_inputs, expected.1);
    assert_ne!(binding_route.dependency_paths, route.dependency_paths);
    assert_ne!(inputs, later_inputs);
}

#[test]
fn shared_negative_retains_watchable_and_builtin_roles_without_losing_inputs() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    std::fs::create_dir_all(&importer).unwrap();
    let mut resolver = PackageRouteResolver::default();
    for (specifier, watchable) in [("missing/widget", true), ("node:fs", false)] {
        let original = resolver.lookup(&importer, specifier, PackageSourceOptions::default());
        let expected = original.clone().into_parts();
        let cached = CachedPackageLookup::from(original);
        assert_eq!(cached.is_watchable_negative(), watchable);
        assert!(cached.route().is_none());
        let (route, inputs) = cached.clone().into_parts();
        assert_eq!(route.as_deref(), expected.0.as_ref());
        assert_eq!(inputs, expected.1);
        assert_eq!(cached.into_parts().1, inputs);
    }
}
