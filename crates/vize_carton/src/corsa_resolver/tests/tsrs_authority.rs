#![cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]

use super::super::{CorsaResolveError, packages::tsrs_platform_package};
use super::{resolve, write_tsrs_install};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn platform_dir(root: &Path) -> PathBuf {
    root.join("node_modules")
        .join(tsrs_platform_package().unwrap())
}

fn hoist_platform(project: &Path, workspace: &Path) -> PathBuf {
    let installed = platform_dir(project);
    let hoisted = platform_dir(workspace);
    fs::create_dir_all(hoisted.parent().unwrap()).unwrap();
    fs::rename(installed, &hoisted).unwrap();
    hoisted
}

#[test]
fn declared_tsrs_accepts_matching_hoisted_platform_metadata() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let project = workspace.join("packages/app");
    write_tsrs_install(&project);
    let hoisted = hoist_platform(&project, &workspace);

    let resolved = resolve(None, Some(&project), &[]).unwrap();

    assert_eq!(resolved, hoisted.join("tsrs").canonicalize().unwrap());
}

#[test]
fn declared_tsrs_rejects_mismatched_hoisted_platform_metadata() {
    let platform_package = tsrs_platform_package().unwrap();
    for (name, version) in [
        ("@tsrs/not-this-platform", "0.18.0"),
        (platform_package, "0.17.0"),
    ] {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path().join("workspace");
        let project = workspace.join("packages/app");
        write_tsrs_install(&project);
        let hoisted = hoist_platform(&project, &workspace);
        fs::write(
            hoisted.join("package.json"),
            format!(r#"{{"name":"{name}","version":"{version}"}}"#),
        )
        .unwrap();

        let error = resolve(None, Some(&project), &[]).unwrap_err();

        assert_eq!(
            error,
            CorsaResolveError::InvalidRuntime {
                package: project.join("node_modules/tsrs"),
            }
        );
    }
}

#[test]
fn declared_tsrs_rejects_a_broken_nearest_platform_link() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let project = workspace.join("packages/app");
    write_tsrs_install(&project);
    hoist_platform(&project, &workspace);
    let nearest = platform_dir(&project);
    fs::create_dir_all(nearest.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(project.join("missing-platform"), nearest).unwrap();

    let error = resolve(None, Some(&project), &[]).unwrap_err();

    assert_eq!(
        error,
        CorsaResolveError::InvalidRuntime {
            package: project.join("node_modules/tsrs"),
        }
    );
}
