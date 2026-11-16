//! Vite+ can transport one evaluated configuration without a temporary file.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn run(root: &Path, args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(root).args(args);
    let Some(stdin) = stdin else {
        return command.output().unwrap();
    };
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn assert_same(left: &Output, right: &Output) {
    assert_eq!(left.status.code(), right.status.code());
    assert_eq!(left.stdout, right.stdout);
    assert_eq!(left.stderr, right.stderr);
}

fn project() -> (tempfile::TempDir, Vec<u8>) {
    let project = tempfile::tempdir().unwrap();
    fs::write(
        project.path().join("App.vue"),
        "<script setup lang=\"ts\">\nconst value = \"x\"\n</script>\n<template><p>{{ value }}</p></template>\n",
    )
    .unwrap();
    let payload = serde_json::to_vec(&serde_json::json!({
        "vue": {
            "basePath": project.path(),
            "formatter": { "singleQuote": true },
            "linter": { "enabled": false },
            "typeChecker": { "enabled": false }
        }
    }))
    .unwrap();
    fs::write(project.path().join("transport.json"), &payload).unwrap();
    (project, payload)
}

#[test]
fn lint_stdin_transport_matches_the_same_explicit_config() {
    let (project, payload) = project();
    let root = project.path();
    let explicit = run(
        root,
        &["lint", "--config", "transport.json", "App.vue"],
        None,
    );
    let transported = run(root, &["lint", "--config-stdin", "App.vue"], Some(&payload));

    assert_same(&explicit, &transported);
    assert_eq!(transported.status.code(), Some(0));
}

#[test]
fn check_stdin_transport_matches_the_same_explicit_config() {
    let (project, payload) = project();
    let root = project.path();
    let explicit = run(
        root,
        &["check", "--config", "transport.json", "App.vue"],
        None,
    );
    let transported = run(
        root,
        &["check", "--config-stdin", "App.vue"],
        Some(&payload),
    );

    assert_same(&explicit, &transported);
    assert_eq!(transported.status.code(), Some(0));
}

#[cfg(feature = "glyph")]
#[test]
fn fmt_stdin_transport_matches_the_same_explicit_config() {
    let (project, payload) = project();
    let root = project.path();
    let explicit = run(
        root,
        &["fmt", "--config", "transport.json", "--check", "App.vue"],
        None,
    );
    let transported = run(
        root,
        &["fmt", "--config-stdin", "--check", "App.vue"],
        Some(&payload),
    );

    assert_same(&explicit, &transported);
    assert_eq!(transported.status.code(), Some(1));
}

#[test]
fn stdin_transport_is_private_and_conflicts_with_public_config_selection() {
    #[cfg(feature = "glyph")]
    let commands = ["check", "lint", "fmt"].as_slice();
    #[cfg(not(feature = "glyph"))]
    let commands = ["check", "lint"].as_slice();
    for &command in commands {
        let help = run(Path::new("."), &[command, "--help"], None);
        assert_eq!(help.status.code(), Some(0));
        let private_help = run(Path::new("."), &[command, "--config-stdin", "--help"], None);
        assert_same(&help, &private_help);

        let conflict = run(
            Path::new("."),
            &[command, "--config-stdin", "--no-config"],
            Some(b"{}"),
        );
        assert_eq!(conflict.status.code(), Some(2));
    }

    #[cfg(unix)]
    {
        let socket_conflict = run(
            Path::new("."),
            &["check", "--config-stdin", "--socket", "unused.sock"],
            Some(b"{}"),
        );
        assert_eq!(socket_conflict.status.code(), Some(2));
    }
}

#[test]
fn check_stdin_config_errors_match_the_explicit_config_path() {
    let project = tempfile::tempdir().unwrap();
    let retired = br#"{ "formatter": { "singleQuote": true } }"#;
    fs::write(project.path().join("retired.json"), retired).unwrap();

    let explicit = run(
        project.path(),
        &["check", "--config", "retired.json", "App.vue"],
        None,
    );
    let transported = run(
        project.path(),
        &["check", "--config-stdin", "App.vue"],
        Some(retired),
    );

    assert_eq!(explicit.status.code(), transported.status.code());
    assert_eq!(explicit.stdout, transported.stdout);
    let config_error = "vize.config.* is no longer supported; move Vue settings to vue: in vite.config.* and TypeScript settings to tsconfig.json";
    let hint =
        "Hint: use `vize check --no-config` to type-check without loading project configuration.\n";
    assert_eq!(
        String::from_utf8(explicit.stderr).unwrap(),
        format!("\x1b[31mError:\x1b[0m failed to parse retired.json: {config_error}\n{hint}")
    );
    assert_eq!(
        String::from_utf8(transported.stderr).unwrap(),
        format!(
            "\x1b[31mError:\x1b[0m failed to parse {}: {config_error}\n{hint}",
            std::env::temp_dir().join(".vize-vp-stdin.json").display()
        )
    );
    assert_eq!(transported.status.code(), Some(2));
}
