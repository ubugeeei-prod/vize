use super::super::{CORSA_ENV_VARS, CorsaResolveError};
use super::{discover_in_walk, platform_suffix, resolve, write_file};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

#[test]
fn nested_vize_optional_platform_compiler_is_the_fallback() {
    let suffix = platform_suffix();
    if suffix.is_empty() {
        return;
    }
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    fs::write(project.join("package.json"), "{}").unwrap();
    let vize = project.join("node_modules").join("vize");
    let package_name = crate::cstr!("@typescript/typescript-{suffix}");
    fs::create_dir_all(&vize).unwrap();
    fs::write(
        vize.join("package.json"),
        &*crate::cstr!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    )
    .unwrap();
    let platform = vize.join("node_modules").join(&package_name);
    fs::create_dir_all(platform.join("lib")).unwrap();
    fs::write(
        platform.join("package.json"),
        &*crate::cstr!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    )
    .unwrap();
    let executable = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    write_file(&platform.join("lib").join(executable));

    let found = discover_in_walk(&[project.to_path_buf()]).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        platform
            .join("lib")
            .join(executable)
            .canonicalize()
            .unwrap()
    );

    pnpm_sibling_link_resolves(&package_name, executable);
}

#[cfg(unix)]
fn pnpm_sibling_link_resolves(package_name: &str, executable: &str) {
    let temp = TempDir::new().unwrap();
    let store = temp.path().join("store").join("node_modules");
    let vize = store.join("vize");
    fs::create_dir_all(&vize).unwrap();
    fs::write(
        vize.join("package.json"),
        &*crate::cstr!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    )
    .unwrap();
    let platform = store.join(package_name);
    fs::create_dir_all(platform.join("lib")).unwrap();
    fs::write(
        platform.join("package.json"),
        &*crate::cstr!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    )
    .unwrap();
    write_file(&platform.join("lib").join(executable));

    let project = temp.path().join("project");
    fs::create_dir_all(project.join("node_modules")).unwrap();
    fs::write(project.join("package.json"), "{}").unwrap();
    std::os::unix::fs::symlink(&vize, project.join("node_modules").join("vize")).unwrap();

    let found = discover_in_walk(&[project]).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        platform
            .join("lib")
            .join(executable)
            .canonicalize()
            .unwrap()
    );
}

#[cfg(not(unix))]
fn pnpm_sibling_link_resolves(_package_name: &str, _executable: &str) {}

// npm hoists Vize's optional package beside Vize. The project itself does not
// declare the platform package, and the platform manifest has no bin entry.
fn hoisted_vize_compiler(root: &Path) -> PathBuf {
    let package_name = crate::cstr!("@typescript/typescript-{}", platform_suffix());
    let vize = root.join("node_modules/vize");
    fs::create_dir_all(&vize).unwrap();
    fs::write(
        vize.join("package.json"),
        &*crate::cstr!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    )
    .unwrap();
    let platform = root.join("node_modules").join(&package_name);
    fs::create_dir_all(&platform).unwrap();
    fs::write(
        platform.join("package.json"),
        &*crate::cstr!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    )
    .unwrap();
    let executable = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    let executable = platform.join("lib").join(executable);
    write_file(&executable);
    executable.canonicalize().unwrap()
}

fn declare_runtime(root: &Path, name: &str, spec: &str) {
    fs::write(
        root.join("package.json"),
        serde_json::to_vec(&serde_json::json!({
            "devDependencies": {"vize":"0.627.0", (name):spec}
        }))
        .unwrap(),
    )
    .unwrap();
}

const ABSENT_RUNTIMES: [(&str, &str); 4] = [
    ("tsrs", "0.20.1-ts7.1.0-beta"),
    ("@typescript/native-preview", "7.0.0"),
    ("tsrs-alias", "npm:tsrs@0.20.1-ts7.1.0-beta"),
    ("preview-alias", "npm:@typescript/native-preview@7.0.0"),
];

#[test]
fn absent_declared_runtime_does_not_hide_the_hoisted_vize_compiler() {
    if platform_suffix().is_empty() {
        return;
    }
    for (name, spec) in ABSENT_RUNTIMES {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        declare_runtime(root, name, spec);
        let expected = hoisted_vize_compiler(root);
        assert!(fs::symlink_metadata(root.join("node_modules").join(name)).is_err());
        assert_eq!(resolve(None, Some(root), &[]), Ok(expected), "{name}");
    }
}

#[test]
fn absent_declared_runtime_without_a_fallback_is_not_found() {
    for (name, spec) in ABSENT_RUNTIMES {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        declare_runtime(root, name, spec);
        assert_eq!(
            resolve(None, Some(root), &[]),
            Err(CorsaResolveError::NotFound),
            "{name}"
        );
    }
}

#[test]
fn present_incomplete_runtime_still_refuses_the_hoisted_vize_fallback() {
    if platform_suffix().is_empty() {
        return;
    }
    for (name, spec) in ABSENT_RUNTIMES {
        for manifest in [None, Some("{"), Some(r#"{"name":"unrelated"}"#)] {
            let temp = TempDir::new().unwrap();
            let root = temp.path();
            declare_runtime(root, name, spec);
            hoisted_vize_compiler(root);
            let package = root.join("node_modules").join(name);
            fs::create_dir_all(&package).unwrap();
            if let Some(manifest) = manifest {
                fs::write(package.join("package.json"), manifest).unwrap();
            }
            assert_eq!(
                resolve(None, Some(root), &[]),
                Err(CorsaResolveError::InvalidRuntime { package }),
                "{name}: {manifest:?}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn broken_nearest_runtime_link_still_refuses_the_hoisted_vize_fallback() {
    if platform_suffix().is_empty() {
        return;
    }
    for (name, spec) in ABSENT_RUNTIMES {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        declare_runtime(root, name, spec);
        hoisted_vize_compiler(root);
        let package = root.join("node_modules").join(name);
        fs::create_dir_all(package.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(root.join("missing-package"), &package).unwrap();
        assert_eq!(
            resolve(None, Some(root), &[]),
            Err(CorsaResolveError::InvalidRuntime { package }),
            "{name}"
        );
    }
}

#[test]
fn valid_project_provider_wins_over_the_hoisted_vize_compiler() {
    if platform_suffix().is_empty() {
        return;
    }
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    declare_runtime(root, "typescript-native", "npm:typescript@7.1.0-beta");
    let bundled = hoisted_vize_compiler(root);
    let package_name = crate::cstr!("@typescript/typescript-{}", platform_suffix());
    let native = root.join("node_modules/typescript-native");
    fs::create_dir_all(&native).unwrap();
    fs::write(
        native.join("package.json"),
        &*crate::cstr!(r#"{{"name":"typescript","version":"7.1.0-beta","optionalDependencies":{{"{package_name}":"7.1.0-beta"}}}}"#),
    )
    .unwrap();
    let platform = native.join("node_modules").join(&package_name);
    fs::create_dir_all(&platform).unwrap();
    fs::write(
        platform.join("package.json"),
        &*crate::cstr!(r#"{{"name":"{package_name}","version":"7.1.0-beta"}}"#),
    )
    .unwrap();
    let executable = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    let selected = platform.join("lib").join(executable);
    write_file(&selected);
    let selected = selected.canonicalize().unwrap();
    assert_ne!(selected, bundled);
    assert_eq!(resolve(None, Some(root), &[]), Ok(selected));
}

#[test]
fn explicit_missing_configuration_and_each_environment_alias_refuse_fallback() {
    if platform_suffix().is_empty() {
        return;
    }
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    declare_runtime(root, "tsrs", "0.20.1-ts7.1.0-beta");
    let bundled = hoisted_vize_compiler(root);
    let missing = root.join("missing-explicit-native");
    assert_eq!(
        resolve(Some(&missing), Some(root), &[]),
        Err(CorsaResolveError::ExplicitNotFound {
            source: "configuration",
            path: missing.clone(),
        })
    );
    for name in CORSA_ENV_VARS {
        assert_eq!(
            resolve(None, Some(root), &[(name, &missing)]),
            Err(CorsaResolveError::ExplicitNotFound {
                source: name,
                path: missing.clone(),
            })
        );
        assert_eq!(
            resolve(Some(&bundled), Some(root), &[(name, &missing)]),
            Ok(bundled.clone())
        );
    }
}
