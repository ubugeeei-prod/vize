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
            r#"{{"name":"typescript","version":"{version}","optionalDependencies":{{"@typescript/typescript-{suffix}":"{version}"}}}}"#
        ),
    )
    .unwrap();
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
    let message = error.to_string();
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
    write_file(&wrapper);
    write_file(&native);

    let resolved = resolve(Some(&wrapper), Some(workspace_root.as_path()), &[]).unwrap();

    assert_eq!(resolved, native.canonicalize().unwrap());
}

#[test]
fn normalizes_wrapper_to_project_cache_when_native_binary_is_absent() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    let cache = root.join(".cache").join("tsgo");
    write_file(&wrapper);
    write_file(&cache);

    assert_eq!(normalize_corsa_path(&wrapper), cache);
}

#[test]
fn normalize_passes_non_wrapper_paths_through() {
    let path = Path::new("/somewhere/else/corsa");
    assert_eq!(normalize_corsa_path(path), path);
}

#[test]
fn prefers_project_local_cache_before_native_preview() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("workspace");
    let cache = root.join(".cache").join("tsgo");
    let native = root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_file(&cache);
    write_file(&native);

    let resolved = discover_in_walk(&[root.join("packages").join("demo")], false);

    assert_eq!(resolved, Some(cache));
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

    let resolved = discover_in_walk(std::slice::from_ref(&root), false);

    assert_eq!(resolved, Some(typescript_binary.canonicalize().unwrap()));
}

#[test]
fn ignores_typescript_six_when_finding_corsa_runtime() {
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

    let resolved = discover_in_walk(std::slice::from_ref(&root), false);

    assert_eq!(resolved, Some(native_preview_binary));
}

#[test]
fn finds_vize_owned_typescript_runtime_when_project_uses_typescript_six() {
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

    let resolved = discover_in_walk(std::slice::from_ref(&root), false);

    assert_eq!(resolved, Some(owned_binary.canonicalize().unwrap()));
}

#[test]
fn prefers_native_preview_binary_over_node_modules_bin_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    let native = root
        .join("node_modules")
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{}", platform_suffix()))
        .join("lib")
        .join("tsgo");
    write_file(&wrapper);
    write_file(&native);

    let resolved = discover_in_walk(std::slice::from_ref(&root), false);

    assert_eq!(resolved, Some(native));
}

#[test]
fn prefers_workspace_native_preview_over_nested_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let nested = workspace_root.join("packages").join("demo");
    let wrapper = nested.join("node_modules").join(".bin").join("tsgo");
    let native = workspace_root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");
    write_file(&wrapper);
    write_file(&native);

    let resolved = discover_in_walk(&[nested], false);

    assert_eq!(resolved, Some(native));
}

#[test]
fn falls_back_to_node_modules_bin_wrapper_when_no_native_binary_exists() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    write_file(&wrapper);

    let resolved = discover_in_walk(&[root], false);

    assert_eq!(resolved, Some(wrapper));
}

mod package_runtime;
