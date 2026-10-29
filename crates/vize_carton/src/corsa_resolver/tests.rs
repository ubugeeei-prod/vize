use super::{
    CORSA_ENV_VARS, CorsaResolveError, CorsaResolveRequest, discover_in_walk, normalize_corsa_path,
    platform_suffix, resolve_with_env,
};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write_file(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "").unwrap();
}

fn write_typescript_manifest(path: &Path, suffix: &str, version: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        &*crate::cstr!(
            r#"{{"name":"typescript","version":"{version}","bin":{{"tsc":"bin/tsc"}},"optionalDependencies":{{"@typescript/typescript-{suffix}":"{version}"}}}}"#
        ),
    )
    .unwrap();
    let project = path.parent().unwrap().parent().unwrap().parent().unwrap();
    if !project.join("package.json").exists() {
        fs::write(
            project.join("package.json"),
            &*crate::cstr!(r#"{{"devDependencies":{{"typescript":"{version}"}}}}"#),
        )
        .unwrap();
    }
    let package = path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"));
    fs::create_dir_all(&package).unwrap();
    fs::write(
        package.join("package.json"),
        &*crate::cstr!(r#"{{"name":"@typescript/typescript-{suffix}","version":"{version}"}}"#),
    )
    .unwrap();
}

fn write_native_preview_packages(root: &Path, version: &str) -> PathBuf {
    let suffix = platform_suffix();
    let meta = root.join("node_modules/@typescript/native-preview");
    let platform = root
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"));
    fs::create_dir_all(&meta).unwrap();
    fs::create_dir_all(&platform).unwrap();
    fs::write(
        meta.join("package.json"),
        &*crate::cstr!(
            r#"{{"name":"@typescript/native-preview","version":"{version}","optionalDependencies":{{"@typescript/native-preview-{suffix}":"{version}"}}}}"#
        ),
    )
    .unwrap();
    fs::write(
        platform.join("package.json"),
        &*crate::cstr!(r#"{{"name":"@typescript/native-preview-{suffix}","version":"{version}"}}"#),
    )
    .unwrap();
    let binary = platform.join("lib/tsgo");
    write_file(&binary);
    binary
}

#[cfg(unix)]
fn write_tsrs_install(root: &Path) -> (PathBuf, PathBuf) {
    let platform_package = super::packages::tsrs_platform_package().unwrap();
    let wrapper = root.join("node_modules").join(".bin").join("tsrs");
    let package = root.join("node_modules").join("tsrs").join("package.json");
    let native = root
        .join("node_modules")
        .join(platform_package)
        .join("tsrs");
    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    write_file(&native);
    fs::create_dir_all(package.parent().unwrap()).unwrap();
    fs::write(
        package,
        &*crate::cstr!(
            r#"{{"name":"tsrs","bin":{{"tsrs":"bin/tsrs.js"}},"optionalDependencies":{{"{platform_package}":"0.18.0"}}}}"#
        ),
    )
    .unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"tsrs":"0.18.0"}}"#,
    )
    .unwrap();
    let launcher = root.join("node_modules/tsrs/bin/tsrs.js");
    write_file(&launcher);
    std::os::unix::fs::symlink(&launcher, &wrapper).unwrap();
    (wrapper, native)
}

fn resolve(
    explicit_path: Option<&Path>,
    project_root: Option<&Path>,
    env: &[(&str, &Path)],
) -> Result<PathBuf, CorsaResolveError> {
    let request = CorsaResolveRequest {
        explicit_path,
        project_root,
    };
    resolve_with_env(request, |name| {
        env.iter()
            .find(|(env_name, _)| *env_name == name)
            .map(|(_, path)| OsString::from(path.as_os_str()))
    })
}

#[test]
fn explicit_path_wins_over_env_vars() {
    let temp_dir = TempDir::new().unwrap();
    let explicit = temp_dir.path().join("explicit").join("corsa");
    let from_env = temp_dir.path().join("env").join("corsa");
    write_file(&explicit);
    write_file(&from_env);

    let resolved = resolve(Some(&explicit), None, &[("CORSA_PATH", from_env.as_path())]).unwrap();

    assert_eq!(resolved, explicit.canonicalize().unwrap());
}

#[test]
fn env_vars_resolve_in_documented_precedence_order() {
    let temp_dir = TempDir::new().unwrap();
    let mut targets = Vec::new();
    for env_name in CORSA_ENV_VARS {
        let target = temp_dir.path().join(env_name).join("corsa");
        write_file(&target);
        targets.push((env_name, target));
    }

    // Drop the highest-precedence var one at a time; the next one wins.
    for first_set in 0..targets.len() {
        let env: Vec<(&str, &Path)> = targets[first_set..]
            .iter()
            .map(|(env_name, path)| (*env_name, path.as_path()))
            .collect();

        let resolved = resolve(None, None, &env).unwrap();

        assert_eq!(
            resolved,
            targets[first_set].1.canonicalize().unwrap(),
            "expected {} to win",
            targets[first_set].0
        );
    }
}

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]
#[test]
fn tsrs_env_resolves_launcher_to_its_native_binary() {
    let temp_dir = TempDir::new().unwrap();
    let (wrapper, native) = write_tsrs_install(temp_dir.path());

    let resolved = resolve(
        None,
        Some(temp_dir.path()),
        &[("TSRS_PATH", wrapper.as_path())],
    )
    .unwrap();

    assert_eq!(resolved, native.canonicalize().unwrap());
}

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]
#[test]
fn discovers_tsrs_native_when_it_is_the_only_project_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let (_, native) = write_tsrs_install(temp_dir.path());

    let resolved = discover_in_walk(&[temp_dir.path().join("src")]);

    assert_eq!(resolved, Some(native.canonicalize().unwrap()));
}

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]
#[test]
fn undeclared_tsrs_requires_explicit_selection() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let (wrapper, native) = write_tsrs_install(root);
    fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();

    assert_eq!(discover_in_walk(&[root.to_path_buf()]), None);
    assert_eq!(
        resolve(Some(&wrapper), Some(root), &[]).unwrap(),
        native.canonicalize().unwrap()
    );
}

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]
#[test]
fn explicit_tsrs_wrapper_does_not_normalize_to_typescript() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let (wrapper, native) = write_tsrs_install(root);
    fs::remove_file(&wrapper).unwrap();
    std::os::unix::fs::symlink(root.join("node_modules/tsrs/bin/tsrs.js"), &wrapper).unwrap();
    write_typescript_manifest(
        &root.join("node_modules/typescript/package.json"),
        suffix,
        "7.0.2",
    );
    write_file(
        &root
            .join("node_modules/@typescript")
            .join(&*crate::cstr!("typescript-{suffix}"))
            .join("lib/tsc"),
    );

    let resolved = resolve(Some(&wrapper), Some(root), &[]).unwrap();

    assert_eq!(resolved, native.canonicalize().unwrap());
}

#[test]
fn explicit_path_must_exist() {
    let temp_dir = TempDir::new().unwrap();
    let missing = temp_dir.path().join("missing-corsa");

    let error = resolve(Some(&missing), None, &[]).unwrap_err();

    assert_eq!(
        error,
        CorsaResolveError::ExplicitNotFound {
            source: "configuration",
            path: missing.clone(),
        }
    );
    let message = crate::cstr!("{error}");
    assert!(message.contains("Configured Corsa executable does not exist"));
    assert!(message.contains("missing-corsa"));
}

#[test]
fn env_var_path_must_exist() {
    let temp_dir = TempDir::new().unwrap();
    let missing = temp_dir.path().join("missing-corsa");

    let error = resolve(None, None, &[("TSGO_PATH", missing.as_path())]).unwrap_err();

    assert_eq!(
        error,
        CorsaResolveError::ExplicitNotFound {
            source: "TSGO_PATH",
            path: missing,
        }
    );
}

#[test]
fn relative_explicit_path_resolves_against_project_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    let explicit = project_root.join("bin").join("tsgo");
    write_file(&explicit);

    let resolved = resolve(
        Some(Path::new("bin/tsgo")),
        Some(project_root.as_path()),
        &[],
    )
    .unwrap();

    assert_eq!(resolved, explicit.canonicalize().unwrap());
}

#[cfg(unix)]
#[test]
fn explicit_wrapper_path_normalizes_to_native_binary() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let wrapper = workspace_root
        .join("packages")
        .join("demo")
        .join("node_modules")
        .join(".bin")
        .join("tsgo");
    let native = workspace_root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_file(&native);
    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&native, &wrapper).unwrap();

    let resolved = resolve(Some(&wrapper), Some(workspace_root.as_path()), &[]).unwrap();

    assert_eq!(resolved, native.canonicalize().unwrap());
}

#[test]
fn cache_and_undeclared_wrapper_require_explicit_selection() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    let cache = root.join(".cache").join("tsgo");
    fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();
    write_file(&wrapper);
    write_file(&cache);

    assert_eq!(discover_in_walk(&[root.to_path_buf()]), None);
    assert_eq!(
        normalize_corsa_path(&wrapper),
        wrapper.canonicalize().unwrap()
    );
    assert_eq!(
        resolve(Some(&cache), Some(root), &[]).unwrap(),
        cache.canonicalize().unwrap()
    );
}

#[test]
fn normalize_passes_non_wrapper_paths_through() {
    let path = Path::new("/somewhere/else/corsa");
    assert_eq!(normalize_corsa_path(path), path);
}

#[test]
fn checkout_cache_never_overrides_a_declared_preview() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("workspace");
    let cache = root.join(".cache").join("tsgo");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let native = write_native_preview_packages(&root, "7.0.0");
    write_file(&cache);

    let starts = [root.join("packages").join("demo")];
    assert_eq!(
        discover_in_walk(&starts),
        Some(native.canonicalize().unwrap())
    );
}

#[test]
fn prefers_typescript_seven_runtime_over_native_preview() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    let manifest = node_modules.join("typescript").join("package.json");
    let typescript_binary = node_modules
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib")
        .join("tsc");
    let native_preview_binary = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_typescript_manifest(&manifest, suffix, "7.0.2");
    write_file(&typescript_binary);
    write_file(&native_preview_binary);

    let resolved = discover_in_walk(std::slice::from_ref(&root));

    assert_eq!(resolved, Some(typescript_binary.canonicalize().unwrap()));
}

#[test]
fn typescript_six_does_not_authorize_an_undeclared_preview() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    let manifest = node_modules.join("typescript").join("package.json");
    let native_preview_binary = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_typescript_manifest(&manifest, suffix, "6.0.3");
    write_file(&native_preview_binary);

    let resolved = discover_in_walk(std::slice::from_ref(&root));

    assert_eq!(resolved, None);
}

#[test]
fn ignores_vize_owned_runtime_and_an_undeclared_preview() {
    let suffix = platform_suffix();
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let node_modules = root.join("node_modules");
    write_typescript_manifest(
        &node_modules.join("typescript").join("package.json"),
        suffix,
        "6.0.3",
    );
    let owned_node_modules = node_modules.join("vize").join("node_modules");
    let owned_manifest = owned_node_modules.join("typescript").join("package.json");
    let owned_binary = owned_node_modules
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib")
        .join("tsc");
    let native_preview_binary = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_typescript_manifest(&owned_manifest, suffix, "7.0.2");
    write_file(&owned_binary);
    write_file(&native_preview_binary);

    let resolved = discover_in_walk(std::slice::from_ref(&root));

    assert_eq!(resolved, None);
}

#[test]
fn prefers_native_preview_binary_over_node_modules_bin_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0","provider":"1.0.0"}}"#,
    )
    .unwrap();
    let native = write_native_preview_packages(&root, "7.0.0");
    let provider = root.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(std::slice::from_ref(&root));

    assert_eq!(resolved, Some(native.canonicalize().unwrap()));
}

#[test]
fn prefers_workspace_native_preview_over_nested_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let nested = workspace_root.join("packages").join("demo");
    let wrapper = nested.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&workspace_root).unwrap();
    fs::write(
        workspace_root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let native = write_native_preview_packages(&workspace_root, "7.0.0");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("package.json"),
        r#"{"devDependencies":{"provider":"1.0.0"}}"#,
    )
    .unwrap();
    let provider = nested.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(&[nested]);

    assert_eq!(resolved, Some(native.canonicalize().unwrap()));
}

#[test]
fn falls_back_to_node_modules_bin_wrapper_when_no_native_binary_exists() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"provider":"1.0.0"}}"#,
    )
    .unwrap();
    let provider = root.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(&[root]);

    assert_eq!(resolved, Some(wrapper));
}

mod package_runtime;
mod project_packages;
mod provider_authority;
