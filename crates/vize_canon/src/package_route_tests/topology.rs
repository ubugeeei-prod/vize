#![allow(clippy::disallowed_methods)]

use super::super::{PackageRouteResolver, PackageSourceOptions};

#[test]
fn self_reference_and_private_imports_share_manifest_routing() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    let source = package.join("src");
    std::fs::create_dir_all(source.join("features")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r##"{
  "name": "@scope/pkg",
  "exports": { "./features/*": { "types": "./src/features/*.vue" } },
  "imports": { "#widget": { "types": "./src/Widget.vue" } }
}"##,
    )
    .unwrap();
    let feature = source.join("features/alpha.vue");
    let widget = source.join("Widget.vue");
    std::fs::write(&feature, "<template />\n").unwrap();
    std::fs::write(&widget, "<template />\n").unwrap();
    let mut resolver = PackageRouteResolver::default();
    let export = resolver
        .resolve(
            &source,
            "@scope/pkg/features/alpha",
            PackageSourceOptions::default(),
        )
        .unwrap();
    let private = resolver
        .resolve(&source, "#widget", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(
        export.unambiguous_source_path(),
        Some(&feature.canonicalize().unwrap())
    );
    assert_eq!(
        private.unambiguous_source_path(),
        Some(&widget.canonicalize().unwrap())
    );
    assert!(export.workspace_source && private.workspace_source);
}

#[test]
fn private_imports_retain_external_package_topology() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("node_modules/@scope/barrel");
    let component = root.path().join("node_modules/@scope/component");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::create_dir_all(&component).unwrap();
    std::fs::write(
        package.join("package.json"),
        r##"{
  "name": "@scope/barrel",
  "imports": { "#component": "@scope/component" }
}"##,
    )
    .unwrap();
    std::fs::write(
        component.join("package.json"),
        r#"{ "name": "@scope/component", "exports": "./Component.vue" }"#,
    )
    .unwrap();
    let source = component.join("Component.vue");
    std::fs::write(&source, "<template />\n").unwrap();
    let route = PackageRouteResolver::default()
        .resolve(&package, "#component", PackageSourceOptions::default())
        .unwrap();
    assert!(route.source_paths.is_empty());
    assert_eq!(route.nested_routes.len(), 1);
    assert_eq!(
        route.nested_routes[0].unambiguous_source_path(),
        Some(&source.canonicalize().unwrap())
    );
}

#[test]
fn self_reference_wins_over_a_nested_install_with_the_same_name() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    let source = package.join("src");
    let nested = source.join("node_modules/@scope/pkg");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/pkg","exports":"./src/self.ts"}"#,
    )
    .unwrap();
    std::fs::write(source.join("self.ts"), "export {};\n").unwrap();
    std::fs::write(
        nested.join("package.json"),
        r#"{"name":"@scope/pkg","exports":"./nested.ts"}"#,
    )
    .unwrap();
    std::fs::write(nested.join("nested.ts"), "export {};\n").unwrap();
    let route = PackageRouteResolver::default()
        .resolve(&source, "@scope/pkg", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(
        route.unambiguous_source_path(),
        Some(&source.join("self.ts").canonicalize().unwrap())
    );
}
