#![allow(clippy::disallowed_methods)]

use std::path::Path;

use serde_json::json;

use super::{
    PackageRequest, PackageRouteResolver, PackageSourceOptions, collect_request_targets,
    collect_targets,
};

#[test]
fn parses_scoped_and_unscoped_package_subpaths() {
    let scoped = PackageRequest::parse("@scope/pkg/feature").unwrap();
    assert_eq!(scoped.package, "@scope/pkg");
    assert_eq!(scoped.subpath, Some("feature"));
    let plain = PackageRequest::parse("pkg/feature").unwrap();
    assert_eq!(plain.package, "pkg");
    assert_eq!(plain.subpath, Some("feature"));
}

#[test]
fn prefers_the_most_specific_export_pattern() {
    let mappings = json!({
        "./*": "./fallback/*.ts",
        "./features/*": "./specific/*.ts"
    });
    let mut candidates = Vec::new();
    collect_request_targets(
        &mappings,
        "./features/alpha",
        Path::new("/pkg"),
        &mut candidates,
    );
    assert_eq!(candidates, vec![Path::new("/pkg/specific/alpha.ts")]);
}

#[test]
fn package_targets_cannot_escape_or_alias_the_package_root() {
    for target in ["../outside.ts", "/outside.ts", "external-package"] {
        let mut candidates = Vec::new();
        collect_targets(&json!(target), Path::new("/pkg"), None, &mut candidates);
        assert!(candidates.is_empty(), "accepted invalid target {target}");
    }
}

#[test]
fn exports_are_authoritative_for_hidden_subpaths() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    let source = package.join("src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/pkg","exports":{".":"./src/index.ts"}}"#,
    )
    .unwrap();
    std::fs::write(source.join("index.ts"), "export {}\n").unwrap();
    std::fs::write(source.join("hidden.ts"), "export {}\n").unwrap();
    let resolved = PackageRouteResolver::default().resolve(
        &source,
        "@scope/pkg/hidden",
        PackageSourceOptions::default(),
    );
    assert_eq!(resolved, None);
}

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

    assert_eq!(export.source_path, feature.canonicalize().unwrap());
    assert_eq!(private.source_path, widget.canonicalize().unwrap());
    assert!(export.workspace_source && private.workspace_source);
}

#[test]
fn installed_dependencies_remain_native_package_sources() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("src");
    let package = root.path().join("node_modules/package");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"package","exports":"./index.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.ts"), "export {}\n").unwrap();
    let route = PackageRouteResolver::default()
        .resolve(&source, "package", PackageSourceOptions::default())
        .unwrap();
    assert!(!route.workspace_source);
    assert_eq!(
        route.source_path,
        package.join("index.ts").canonicalize().unwrap()
    );
}

#[test]
fn runtime_export_prefers_its_declaration_sidecar() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("src");
    let package = root.path().join("node_modules/runtime-package");
    let runtime = package.join("dist/index.mjs");
    let declaration = package.join("dist/index.d.mts");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(runtime.parent().unwrap()).unwrap();
    std::fs::write(&runtime, "export const value = 1\n").unwrap();
    std::fs::write(&declaration, "export declare const value: number\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":{"import":"./dist/index.mjs"}}}"#,
    )
    .unwrap();
    let route = PackageRouteResolver::default()
        .resolve(&source, "runtime-package", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(route.source_path, declaration.canonicalize().unwrap());
}

#[test]
fn export_arrays_fall_back_but_cannot_escape_the_package() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("src");
    let package = root.path().join("node_modules/fallback-package");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(package.join("dist/valid.d.ts"), "export {}\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{
  "exports": {
    ".": ["./dist/missing.d.ts", "./dist/valid.d.ts"],
    "./escape": "./../escape.d.ts"
  }
}"#,
    )
    .unwrap();
    let mut resolver = PackageRouteResolver::default();

    let route = resolver
        .resolve(&source, "fallback-package", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(
        route.source_path,
        package.join("dist/valid.d.ts").canonicalize().unwrap()
    );
    assert!(
        resolver
            .resolve(
                &source,
                "fallback-package/escape",
                PackageSourceOptions::default(),
            )
            .is_none()
    );
}

#[test]
fn null_and_unknown_export_conditions_do_not_fall_through() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("src");
    let package = root.path().join("node_modules/conditional-package");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(package.join("dist/runtime.mjs"), "export {}\n").unwrap();
    std::fs::write(package.join("dist/browser.d.ts"), "export {}\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{
  "exports": {
    ".": { "types": null, "import": "./dist/runtime.mjs" },
    "./browser": { "browser": "./dist/browser.d.ts" }
  }
}"#,
    )
    .unwrap();
    let mut resolver = PackageRouteResolver::default();

    assert!(
        resolver
            .resolve(
                &source,
                "conditional-package",
                PackageSourceOptions::new(true, true),
            )
            .is_none()
    );
    assert!(
        resolver
            .resolve(
                &source,
                "conditional-package/browser",
                PackageSourceOptions::default(),
            )
            .is_none()
    );
}

#[test]
fn missing_explicit_vue_target_is_retained_for_create_invalidation() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    let source = package.join("src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"pkg","exports":{".":"./src/Future.vue"}}"#,
    )
    .unwrap();
    let route = PackageRouteResolver::default()
        .resolve(&source, "pkg", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(route.source_path, route.package_root.join("src/Future.vue"));
    assert!(route.invalidation_paths().contains(&route.manifest_path));
}

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
