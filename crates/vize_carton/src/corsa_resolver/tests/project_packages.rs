use super::super::{CorsaResolveError, discover_in_walk, discovery_starts, platform_suffix};
use super::{resolve, write_file, write_typescript_manifest};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn native_package(root: &Path, version: &str) -> PathBuf {
    let modules = root.join("node_modules");
    write_typescript_manifest(
        &modules.join("typescript/package.json"),
        platform_suffix(),
        version,
    );
    let binary = modules
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{}", platform_suffix()))
        .join("lib/tsc");
    write_file(&binary);
    binary.canonicalize().unwrap()
}

#[test]
fn selected_project_does_not_add_an_unrelated_working_directory() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("selected");
    let cwd = temp.path().join("unrelated");
    let own = native_package(&project, "7.1.0-beta");
    native_package(&cwd, "7.0.2");
    let starts = discovery_starts(Some(&project), Some(&cwd));
    assert_eq!(starts, vec![project]);
    assert_eq!(discover_in_walk(&starts, false), Some(own));
    assert_eq!(discovery_starts(None, Some(&cwd)), vec![cwd.clone()]);
    assert_eq!(
        discovery_starts(Some(Path::new("nested")), Some(&cwd)),
        vec![cwd.join("nested")]
    );
}

#[test]
fn missing_relative_override_never_uses_a_same_named_cwd_file() {
    let temp = TempDir::new().unwrap();
    // Cargo.toml exists in the test runner's checkout, outside this project.
    let error = resolve(Some(Path::new("Cargo.toml")), Some(temp.path()), &[]).unwrap_err();
    assert_eq!(
        error,
        CorsaResolveError::ExplicitNotFound {
            source: "configuration",
            path: temp.path().join("Cargo.toml"),
        }
    );
    let valid = temp.path().join("native");
    write_file(&valid);
    assert!(matches!(
        resolve(
            None,
            Some(temp.path()),
            &[("CORSA_PATH", Path::new("missing")), ("TSGO_PATH", &valid)]
        ),
        Err(CorsaResolveError::ExplicitNotFound {
            source: "CORSA_PATH",
            ..
        })
    ));
}

#[cfg(unix)]
#[test]
fn declared_native_alias_follows_its_symlinked_platform_dependency() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("package.json"), r#"{"devDependencies":{"typescript":"6.0.3","typescript-native":"npm:typescript@7.1.0-beta"}}"#).unwrap();
    native_package(root, "6.0.3");
    let store = root.join("node_modules/.pnpm/typescript@7.1.0-beta");
    let expected = native_package(&store, "7.1.0-beta");
    std::os::unix::fs::symlink(
        store.join("node_modules/typescript"),
        root.join("node_modules/typescript-native"),
    )
    .unwrap();
    let stale = root.join("node_modules/vize");
    native_package(&stale, "7.0.2");
    assert_eq!(
        discover_in_walk(&[root.to_path_buf()], false),
        Some(expected)
    );
}

#[test]
fn platform_version_must_match_the_selected_meta_package() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let binary = native_package(root, "7.1.0-beta");
    let package = binary
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("package.json");
    fs::write(
        package,
        &*crate::cstr!(
            r#"{{"name":"@typescript/typescript-{}","version":"7.0.2"}}"#,
            platform_suffix()
        ),
    )
    .unwrap();
    assert_eq!(discover_in_walk(&[root.to_path_buf()], false), None);
}

#[test]
fn typescript_six_or_an_unrelated_alias_cannot_claim_native_platform_files() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(
        root.join("package.json"),
        r#"{"dependencies":{"native-alias":"npm:unrelated@7.1.0-beta"}}"#,
    )
    .unwrap();
    let binary = native_package(root, "6.0.3");
    fs::create_dir_all(root.join("node_modules/native-alias")).unwrap();
    fs::write(root.join("node_modules/native-alias/package.json"), &*crate::cstr!(r#"{{"name":"unrelated","version":"7.1.0-beta","optionalDependencies":{{"@typescript/typescript-{}":"7.1.0-beta"}}}}"#, platform_suffix())).unwrap();
    assert!(binary.is_file());
    write_file(&root.join("node_modules/.bin/tsc"));
    assert_eq!(discover_in_walk(&[root.to_path_buf()], false), None);
}

#[test]
fn project_preview_is_not_overridden_by_an_ancestor_native_version() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("packages/app");
    native_package(temp.path(), "7.0.2");
    let own = nested.join("node_modules/@typescript/native-preview/lib/tsgo");
    write_file(&own);
    assert_eq!(discover_in_walk(&[nested], false), Some(own));
}

#[test]
fn missing_runtime_message_is_actionable_without_a_bare_executable_fallback() {
    let message = crate::cstr!("{}", CorsaResolveError::NotFound);
    assert_eq!(
        message.as_str(),
        "No native TypeScript/tsgo executable found. Install a native TypeScript runtime in the selected project, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to its executable."
    );
}
