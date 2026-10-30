use super::super::{CorsaResolveError, discover_in_walk, platform_suffix};
use super::{resolve, write_file, write_native_preview_packages, write_typescript_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn resolves_declared_native_preview_platform_package() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let expected = write_native_preview_packages(&root, "7.0.0");

    assert_eq!(
        discover_in_walk(&[root]),
        Some(expected.canonicalize().unwrap())
    );
}

// npm's Windows package contains `lib/tsgo.exe`; the resolver must not choose
// a POSIX `.bin/tsgo` shim when the native executable is available.
#[test]
fn resolves_declared_preview_exe_before_its_bin_wrapper() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let extensionless = write_native_preview_packages(&root, "7.0.0");
    fs::remove_file(&extensionless).unwrap();
    let expected = extensionless.with_extension("exe");
    write_file(&expected);
    write_file(&root.join("node_modules/.bin/tsgo"));

    assert_eq!(
        discover_in_walk(&[root]),
        Some(expected.canonicalize().unwrap())
    );
}

#[cfg(unix)]
#[test]
fn resolves_declared_preview_through_a_pnpm_link() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let store = root.join("node_modules/.pnpm/native-preview@7.0.0");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let expected = write_native_preview_packages(&store, "7.0.0");
    let scope = root.join("node_modules/@typescript");
    fs::create_dir_all(&scope).unwrap();
    std::os::unix::fs::symlink(
        store.join("node_modules/@typescript/native-preview"),
        scope.join("native-preview"),
    )
    .unwrap();

    assert_eq!(
        discover_in_walk(&[root]),
        Some(expected.canonicalize().unwrap())
    );
}

#[test]
fn undeclared_packages_and_checkout_caches_are_never_selected() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();
    let preview = write_native_preview_packages(&root, "7.0.0");
    let cache = root.join("ref/typescript-go/built/local/tsgo");
    write_file(&cache);

    assert_eq!(discover_in_walk(std::slice::from_ref(&root)), None);
    assert_eq!(
        resolve(Some(&preview), Some(&root), &[]).unwrap(),
        preview.canonicalize().unwrap()
    );
    assert_eq!(
        resolve(Some(&cache), Some(&root), &[]).unwrap(),
        cache.canonicalize().unwrap()
    );
}

#[test]
fn declared_typescript_seven_wins_over_an_undeclared_preview() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let manifest = root.join("node_modules/typescript/package.json");
    write_typescript_manifest(&manifest, suffix, "7.0.2");
    let expected = root
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib/tsc");
    write_file(&expected);
    write_native_preview_packages(&root, "7.0.0");

    assert_eq!(
        discover_in_walk(&[root]),
        Some(expected.canonicalize().unwrap())
    );
}

#[test]
fn typescript_six_can_coexist_with_a_declared_preview_runtime() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    write_typescript_manifest(
        &root.join("node_modules/typescript/package.json"),
        suffix,
        "6.0.3",
    );
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"typescript":"6.0.3","@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let expected = write_native_preview_packages(&root, "7.0.0");

    assert_eq!(
        discover_in_walk(&[root]),
        Some(expected.canonicalize().unwrap())
    );
}

#[test]
fn bundled_vize_runtime_is_the_fallback_before_an_explicit_unlinked_store() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let install = temp.path().join("install");
    let project = install.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("package.json"), r#"{"private":true}"#).unwrap();
    let unlinked = install
        .join("node_modules/.pnpm")
        .join(&*crate::cstr!("@typescript+native-preview-{suffix}@7.0.0"))
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib/tsgo");
    write_file(&unlinked);
    let owned = install.join("node_modules/vize");
    write_typescript_manifest(
        &owned.join("node_modules/typescript/package.json"),
        suffix,
        "7.0.2",
    );
    let owned_binary = owned
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib/tsc");
    write_file(&owned_binary);

    assert_eq!(
        discover_in_walk(std::slice::from_ref(&project)),
        Some(owned_binary.canonicalize().unwrap())
    );
    assert_eq!(
        resolve(Some(&unlinked), Some(&project), &[]).unwrap(),
        unlinked.canonicalize().unwrap()
    );
}

#[test]
fn an_incomplete_declared_preview_is_an_invalid_runtime() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let package = root.join("node_modules/@typescript/native-preview");
    fs::create_dir_all(&package).unwrap();
    fs::write(
        package.join("package.json"),
        &*crate::cstr!(
            r#"{{"name":"@typescript/native-preview","version":"7.0.0","optionalDependencies":{{"@typescript/native-preview-{suffix}":"7.0.0"}}}}"#
        ),
    )
    .unwrap();

    assert!(matches!(
        resolve(None, Some(&root), &[]),
        Err(CorsaResolveError::InvalidRuntime { package: invalid }) if invalid == package
    ));
}
