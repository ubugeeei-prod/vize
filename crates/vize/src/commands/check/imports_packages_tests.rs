use super::{
    PackageImportResolver, PackageRequest, collect_export_targets, collect_map_request_targets,
};
use crate::commands::check::{imports::ImportFileOptions, path_cache::CanonicalPathCache};
use serde_json::json;
use std::path::Path;

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
    collect_map_request_targets(
        &mappings,
        "./features/alpha",
        Path::new("/pkg"),
        &mut candidates,
    );
    assert_eq!(candidates, vec![Path::new("/pkg/specific/alpha.ts")]);
}

#[test]
fn package_exports_do_not_leak_hidden_subpaths() {
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
    let resolved = PackageImportResolver::default().resolve(
        &source,
        "@scope/pkg/hidden",
        &mut CanonicalPathCache::default(),
        ImportFileOptions::default(),
    );
    assert_eq!(resolved, None);
}

#[test]
fn installed_dependencies_do_not_enter_the_authored_graph() {
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
    let resolved = PackageImportResolver::default().resolve(
        &source,
        "package",
        &mut CanonicalPathCache::default(),
        ImportFileOptions::default(),
    );
    assert_eq!(resolved, None);
}

#[test]
fn package_targets_cannot_escape_or_alias_the_package_root() {
    for target in ["../outside.ts", "/outside.ts", "external-package"] {
        let mut candidates = Vec::new();
        collect_export_targets(&json!(target), Path::new("/pkg"), None, &mut candidates);
        assert!(candidates.is_empty(), "accepted invalid target {target}");
    }
}
