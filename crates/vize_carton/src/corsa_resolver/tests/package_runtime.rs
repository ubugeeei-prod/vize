use super::super::{discover_in_walk, platform_suffix};
use super::{write_file, write_typescript_manifest};
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
fn falls_back_to_typescript_seven_bin_wrapper_when_platform_package_missing() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    let wrapper = node_modules.join(".bin").join("tsc");

    write_typescript_manifest(
        &node_modules.join("typescript").join("package.json"),
        suffix,
        "7.0.2",
    );
    write_file(&wrapper);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(wrapper));
}

#[test]
fn ignores_typescript_six_bin_wrapper() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");

    write_typescript_manifest(
        &node_modules.join("typescript").join("package.json"),
        suffix,
        "6.0.3",
    );
    write_file(&node_modules.join(".bin").join("tsc"));

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, None);
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

/// A prepared upstream workspace may bring its own older native preview.
/// Its API can lack `-async`, while Vize's enclosing install owns the current
/// runtime. Keep upstream's frozen package closure without selecting that API.
#[test]
fn ancestor_current_runtime_precedes_nested_router_preview() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let install = temp.path().join("vize-install");
    let fixture = install.join("target/realworld/vue-router");
    let playground = fixture.join("packages/playground");
    let preview_version = "7.0.0-dev.20260106.1";
    let legacy = fixture
        .join("node_modules/.pnpm")
        .join(&*crate::cstr!(
            "@typescript+native-preview-{suffix}@{preview_version}"
        ))
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
        .join("lib/tsgo");
    let router_manifest = fixture.join("packages/router/package.json");
    fs::create_dir_all(router_manifest.parent().unwrap()).unwrap();
    fs::write(&router_manifest, &*crate::cstr!(
        r#"{{"name":"vue-router","devDependencies":{{"@typescript/native-preview":"{preview_version}"}}}}"#
    )).unwrap();
    let owned_modules = install.join("node_modules/vize/node_modules");
    let current = owned_modules
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib/tsc");
    write_file(&legacy);
    write_file(&playground.join("tsconfig.json"));
    write_typescript_manifest(
        &owned_modules.join("typescript/package.json"),
        suffix,
        "7.0.2",
    );
    write_file(&current);

    assert_eq!(
        discover_in_walk(std::slice::from_ref(&playground), false),
        Some(current.canonicalize().unwrap())
    );
    // Explicit configuration and the existing environment override still own
    // the selection, even when that executable is an older preview.
    assert_eq!(
        super::resolve(Some(&legacy), Some(&playground), &[]).unwrap(),
        legacy.canonicalize().unwrap()
    );
    assert_eq!(
        super::resolve(None, Some(&playground), &[("CORSA_PATH", &legacy)]).unwrap(),
        legacy.canonicalize().unwrap()
    );
    fs::remove_file(&current).unwrap();
    assert_eq!(discover_in_walk(&[playground], false), Some(legacy));
}

#[test]
fn current_runtime_order_remains_nearest_then_start_root() {
    let suffix = platform_suffix();
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("install");
    let nested = root.join("project");
    let other = temp.path().join("other-install");
    let make_current = |directory: &std::path::Path| {
        let modules = directory.join("node_modules");
        let executable = modules
            .join("@typescript")
            .join(&*crate::cstr!("typescript-{suffix}"))
            .join("lib/tsc");
        write_typescript_manifest(&modules.join("typescript/package.json"), suffix, "7.0.2");
        write_file(&executable);
        executable.canonicalize().unwrap()
    };
    let outer = make_current(&root);
    let nearest = make_current(&nested);
    let independent = make_current(&other);
    assert_eq!(
        discover_in_walk(&[nested.clone(), other.clone()], false),
        Some(nearest)
    );
    assert_eq!(
        discover_in_walk(&[other, nested.clone()], false),
        Some(independent)
    );
    fs::remove_file(
        nested
            .join("node_modules")
            .join("@typescript")
            .join(&*crate::cstr!("typescript-{suffix}"))
            .join("lib/tsc"),
    )
    .unwrap();
    assert_eq!(discover_in_walk(&[nested], false), Some(outer));
}

#[test]
fn legacy_only_order_remains_nearest_then_start_root() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("install");
    let nested = root.join("project");
    let other = temp.path().join("other-install");
    let path = |directory: &std::path::Path| {
        directory.join("node_modules/@typescript/native-preview/lib/tsgo")
    };
    for directory in [&root, &nested, &other] {
        write_file(&path(directory));
    }
    assert_eq!(
        discover_in_walk(&[nested.clone(), other.clone()], false),
        Some(path(&nested))
    );
    assert_eq!(
        discover_in_walk(&[other.clone(), nested.clone()], false),
        Some(path(&other))
    );
    fs::remove_file(path(&nested)).unwrap();
    assert_eq!(discover_in_walk(&[nested], false), Some(path(&root)));
}
