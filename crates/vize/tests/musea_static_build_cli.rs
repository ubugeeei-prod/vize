//! Regression coverage for the Vite child process used by `vize musea --build`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_project(root: &Path) {
    fs::write(
        root.join("package.json"),
        r#"{
  "devDependencies": {
    "@vizejs/vite-plugin-musea": "0.315.1"
  }
}"#,
    )
    .unwrap();
    fs::write(
        root.join("vite.config.mts"),
        r#"import { musea } from "@vizejs/vite-plugin-musea";

export default { plugins: [...musea({ storybookCompat: true })] };
"#,
    )
    .unwrap();
}

#[cfg(unix)]
fn write_vite_bin(root: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin = root.join("node_modules/.bin/vite");
    fs::create_dir_all(bin.parent().unwrap()).unwrap();
    fs::write(
        &bin,
        r#"#!/bin/sh
if [ "$1" != "build" ]; then
  exit 9
fi
if [ "${VIZE_MUSEA_STATIC_BUILD:-}" != "1" ]; then
  printf 'Could not resolve entry module "index.html"\n' >&2
  exit 1
fi
mkdir -p dist/__musea__
printf '<!doctype html><title>Musea</title>\n' > dist/__musea__/index.html
"#,
    )
    .unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

#[cfg(windows)]
fn write_vite_bin(root: &Path) -> PathBuf {
    let bin = root.join("node_modules/.bin/vite.cmd");
    fs::create_dir_all(bin.parent().unwrap()).unwrap();
    fs::write(
        &bin,
        r#"@echo off
if not "%1"=="build" exit /b 9
if not "%VIZE_MUSEA_STATIC_BUILD%"=="1" exit /b 1
mkdir dist\__musea__
echo ^<!doctype html^>^<title^>Musea^</title^> > dist\__musea__\index.html
"#,
    )
    .unwrap();
    bin
}

#[test]
fn musea_build_sets_static_mode_for_the_vite_child() {
    let project = tempfile::tempdir().unwrap();
    write_project(project.path());
    let vite_bin = write_vite_bin(project.path());
    assert!(vite_bin.is_file());
    assert!(!project.path().join("index.html").exists());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["musea", "--build"])
        .current_dir(project.path())
        .env("VIZE_MUSEA_STATIC_BUILD", "0")
        .output()
        .expect("failed to run vize musea --build");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(
        stderr.contains("env: VIZE_MUSEA_STATIC_BUILD=1"),
        "{stderr}"
    );
    assert!(project.path().join("dist/__musea__/index.html").is_file());
}
