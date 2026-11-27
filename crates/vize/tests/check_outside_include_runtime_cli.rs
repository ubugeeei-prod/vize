#![cfg(all(test, unix))]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! A tsconfig `include` that reaches outside its package widens the
//! materialized project root to a common ancestor, but the native runtime
//! stays the package's own dependency.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
use vize_carton::corsa_resolver::{CORSA_ENV_VARS, platform_suffix};

const FIXTURE: &str = "tests/_fixtures/differential/typechecker/runtime-owner-outside-include";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

fn write_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(workspace_root().join(FIXTURE).join(name)).unwrap()
}

/// `work/app` installs `vize`, whose optional platform compiler is the only
/// runtime it owns; `work/` has no packages. The compiler only records that it
/// was the one launched.
fn write_project(name: &str) -> Option<(PathBuf, PathBuf)> {
    let suffix = platform_suffix();
    if suffix.is_empty() {
        return None;
    }
    let work = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&work);
    let app = work.join("app");
    write_file(&work.join("shared/a.ts"), &fixture("shared-a.ts.txt"));
    write_file(
        &app.join("package.json"),
        r#"{"name":"app","private":true}"#,
    );
    write_file(&app.join("tsconfig.json"), &fixture("tsconfig.json.txt"));
    write_file(&app.join("src/App.vue"), &fixture("App.vue.txt"));

    let package_name = format!("@typescript/typescript-{suffix}");
    let vize = app.join("node_modules/vize");
    write_file(
        &vize.join("package.json"),
        &format!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    );
    let platform = vize.join("node_modules").join(&package_name);
    write_file(
        &platform.join("package.json"),
        &format!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    );
    let runtime = platform.join("lib/tsc");
    write_file(&runtime, "#!/bin/sh\n: > \"$0.used\"\nexit 1\n");
    std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o755)).unwrap();
    Some((work, runtime))
}

fn run_check(app: &Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(app).arg("check").args(args);
    for name in CORSA_ENV_VARS {
        command.env_remove(name);
    }
    command.output().unwrap()
}

fn assert_launched(runtime: &Path, output: &std::process::Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        runtime.with_file_name("tsc.used").is_file(),
        "the package's own runtime was not launched\nstderr:\n{stderr}"
    );
}

#[test]
fn check_with_outside_include_discovers_the_package_runtime() {
    let Some((work, runtime)) = write_project("outside-include-runtime") else {
        return;
    };

    let output = run_check(&work.join("app"), &[]);
    assert_launched(&runtime, &output);

    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn check_with_outside_include_anchors_a_relative_corsa_path_at_the_package() {
    let Some((work, runtime)) = write_project("outside-include-relative-runtime") else {
        return;
    };
    let app = work.join("app");
    let relative = runtime.strip_prefix(&app).unwrap().display().to_string();

    let output = run_check(&app, &["--corsa-path", &relative]);
    assert_launched(&runtime, &output);

    let _ = std::fs::remove_dir_all(&work);
}
