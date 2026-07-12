#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn lint_fix_write_failure_is_nonzero_and_preserves_source() {
    let project = tempfile::tempdir().unwrap();
    let source_path = project.path().join("App.vue");
    let original = r#"<template><button v-on:click="save">Save</button></template>"#;
    fs::write(&source_path, original).unwrap();

    // A direct write can still truncate an existing writable file in a
    // read-only directory; an atomic writer must fail before replacement.
    let original_permissions = fs::metadata(project.path()).unwrap().permissions();
    let mut read_only_directory = original_permissions.clone();
    read_only_directory.set_mode(0o500);
    fs::set_permissions(project.path(), read_only_directory).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--fix", "--no-config"])
        .arg(&source_path)
        .output()
        .unwrap();

    fs::set_permissions(project.path(), original_permissions).unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert_eq!(fs::read_to_string(&source_path).unwrap(), original);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Failed to write"),
        "{output:?}"
    );
}

#[test]
fn fmt_write_preserves_source_permissions() {
    let project = tempfile::tempdir().unwrap();
    let source_path = project.path().join("App.vue");
    let original = "<template><div/></template>";
    fs::write(&source_path, original).unwrap();
    fs::set_permissions(&source_path, fs::Permissions::from_mode(0o640)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["fmt", "--write", "--no-config"])
        .arg(&source_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_ne!(fs::read_to_string(&source_path).unwrap(), original);
    assert_eq!(
        fs::metadata(&source_path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}
