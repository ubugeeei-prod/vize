use std::fs;
use std::process::Command;

#[test]
fn lint_fix_applies_fixable_template_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("App.vue");
    fs::write(
        &file,
        r#"<template><button v-on:click="save">Save</button></template>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--fix", "--no-config"])
        .arg(&file)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        r#"<template><button @click="save">Save</button></template>"#
    );
}
