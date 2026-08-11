#![allow(clippy::disallowed_methods)]

use std::path::Path;

use serde_json::json;

use super::{
    PackageRequest, PackageRouteResolver, PackageSourceOptions, collect_request_targets,
    collect_targets,
};

#[path = "package_route_tests/lifecycle.rs"]
mod lifecycle;
#[path = "package_route_tests/source.rs"]
mod source;
#[path = "package_route_tests/topology.rs"]
mod topology;

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
fn rejects_alias_and_absolute_spellings_as_package_requests() {
    for specifier in [
        "@/components",
        "@/",
        "@scope/",
        "@scope",
        "@",
        "/src/App.vue",
        "C:/src/App.vue",
        r"C:\src\App.vue",
    ] {
        assert!(
            PackageRequest::parse(specifier).is_none(),
            "accepted non-package specifier {specifier}"
        );
    }
    assert_eq!(
        PackageRequest::parse("@scope/package").unwrap().package,
        "@scope/package"
    );
}

#[test]
fn retains_all_matching_export_patterns_for_native_selection() {
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
    assert_eq!(
        candidates,
        vec![
            Path::new("/pkg/fallback/features/alpha.ts"),
            Path::new("/pkg/specific/alpha.ts"),
        ]
    );
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
        route.unambiguous_source_path().unwrap().clone(),
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
    assert_eq!(
        route.unambiguous_source_path().unwrap().clone(),
        declaration.canonicalize().unwrap()
    );
}

#[test]
fn runtime_export_resolves_a_declaration_when_the_runtime_file_is_absent() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("src");
    let package = root.path().join("node_modules/runtime-package");
    let declaration = package.join("dist/index.d.ts");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(declaration.parent().unwrap()).unwrap();
    std::fs::write(&declaration, "export declare const value: number\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":{"import":"./dist/index.js"}}}"#,
    )
    .unwrap();

    let route = PackageRouteResolver::default()
        .resolve(&source, "runtime-package", PackageSourceOptions::default())
        .unwrap();
    assert_eq!(
        route.unambiguous_source_path().unwrap().clone(),
        declaration.canonicalize().unwrap()
    );
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
        route.unambiguous_source_path().unwrap().clone(),
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
fn conditional_candidates_are_retained_without_vize_owned_priority() {
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

    let root = resolver
        .resolve(
            &source,
            "conditional-package",
            PackageSourceOptions::new(true, true),
        )
        .unwrap();
    assert_eq!(
        root.source_paths,
        vec![package.join("dist/runtime.mjs").canonicalize().unwrap()]
    );
    let browser = resolver
        .resolve(
            &source,
            "conditional-package/browser",
            PackageSourceOptions::default(),
        )
        .unwrap();
    assert_eq!(
        browser.source_paths,
        vec![package.join("dist/browser.d.ts").canonicalize().unwrap()]
    );
    assert_eq!(root.unambiguous_source_path(), root.source_paths.first());
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
    assert_eq!(
        route.unambiguous_source_path().unwrap().clone(),
        route.package_root.join("src/Future.vue")
    );
    assert!(route.invalidation_paths().contains(&route.manifest_path));
}
