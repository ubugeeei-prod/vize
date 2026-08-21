use super::super::{discover_in_walk, platform_suffix};
use super::write_file;
use std::fs;
use tempfile::TempDir;

#[test]
fn resolves_platform_package_from_native_preview_manifest() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    let manifest = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("package.json");
    let platform_binary = node_modules
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib")
        .join("tsgo");

    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    fs::write(
        &manifest,
        &*crate::cstr!(
            r#"{{"name":"@typescript/native-preview","optionalDependencies":{{"@typescript/native-preview-{suffix}":"7.0.0"}}}}"#
        ),
    )
    .unwrap();
    write_file(&platform_binary);

    let resolved = discover_in_walk(&[root], false);

    // Node-style resolution canonicalizes the meta package directory, so
    // compare canonicalized paths (macOS tempdirs live behind a symlink).
    assert_eq!(resolved, Some(platform_binary.canonicalize().unwrap()));
}

// Regression for the native-smoke fresh-install matrix on Windows: npm's
// platform packages ship `lib/tsgo.exe` (no extensionless sibling), and
// `node_modules/.bin/tsgo` is a POSIX sh shim that CreateProcess rejects
// with "%1 is not a valid Win32 application" (os error 193). The resolver
// must find the `.exe` and never fall back to the sh shim.
#[test]
fn resolves_platform_package_exe_binary_over_bin_wrapper() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    let manifest = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("package.json");
    let platform_binary = node_modules
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib")
        .join("tsgo.exe");
    let wrapper = node_modules.join(".bin").join("tsgo");

    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    fs::write(
        &manifest,
        &*crate::cstr!(
            r#"{{"name":"@typescript/native-preview","optionalDependencies":{{"@typescript/native-preview-{suffix}":"7.0.0"}}}}"#
        ),
    )
    .unwrap();
    write_file(&platform_binary);
    write_file(&wrapper);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(platform_binary.canonicalize().unwrap()));
}

#[test]
fn resolves_meta_package_exe_binary() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let native = root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo.exe");
    write_file(&native);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(native));
}

#[cfg(unix)]
#[test]
fn resolves_platform_package_through_pnpm_symlink_layout() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let store_package = root
        .join("node_modules")
        .join(".pnpm")
        .join("@typescript+native-preview@7.0.0")
        .join("node_modules");
    let manifest = store_package
        .join("@typescript")
        .join("native-preview")
        .join("package.json");
    let platform_binary = store_package
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib")
        .join("tsgo");

    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    fs::write(
        &manifest,
        &*crate::cstr!(
            r#"{{"name":"@typescript/native-preview","optionalDependencies":{{"@typescript/native-preview-{suffix}":"7.0.0"}}}}"#
        ),
    )
    .unwrap();
    write_file(&platform_binary);

    let link_parent = root.join("node_modules").join("@typescript");
    fs::create_dir_all(&link_parent).unwrap();
    std::os::unix::fs::symlink(
        store_package.join("@typescript").join("native-preview"),
        link_parent.join("native-preview"),
    )
    .unwrap();

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(platform_binary.canonicalize().unwrap()));
}

#[test]
fn scrapes_pnpm_store_when_meta_package_is_not_linked() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let store_binary = root
        .join("node_modules")
        .join(".pnpm")
        .join(&*crate::cstr!("@typescript+native-preview-{suffix}@7.0.0"))
        .join("node_modules")
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib")
        .join("tsgo");
    write_file(&store_binary);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(store_binary));
}

#[test]
fn scrapes_pnpm_store_when_typescript_platform_package_is_not_linked() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let store_binary = root
        .join("node_modules")
        .join(".pnpm")
        .join(&*crate::cstr!("@typescript+typescript-{suffix}@7.0.2"))
        .join("node_modules")
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib")
        .join("tsc");
    write_file(&store_binary);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(store_binary));
}

#[test]
fn dev_paths_expose_typescript_go_checkout_binaries() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("checkout");
    let built = root
        .join("ref")
        .join("typescript-go")
        .join("built")
        .join("local")
        .join("tsgo");
    write_file(&built);

    assert_eq!(
        discover_in_walk(std::slice::from_ref(&root), true),
        Some(built)
    );
    assert_eq!(discover_in_walk(&[root], false), None);
}

#[test]
fn dev_paths_expose_sibling_corsa_bind_cache() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("workspace");
    let nested = root.join("packages").join("demo");
    let sibling_cache = temp_dir
        .path()
        .join("corsa-bind")
        .join(".cache")
        .join("tsgo");
    fs::create_dir_all(&nested).unwrap();
    write_file(&sibling_cache);

    assert_eq!(
        discover_in_walk(std::slice::from_ref(&nested), true),
        Some(sibling_cache)
    );
    assert_eq!(discover_in_walk(&[nested], false), None);
}
