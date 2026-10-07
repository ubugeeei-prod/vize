#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
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

#[test]
fn lint_fix_converges_overlapping_binding_fixes_and_is_idempotent() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.fixed.vue.txt"
    );
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("TitleBox.vue");
    fs::write(&file, source).unwrap();

    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args([
                "lint",
                "--fix",
                "--no-config",
                "--preset",
                "opinionated",
                "--format",
                "plain",
                "--help-level",
                "none",
            ])
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
        assert!(output.stdout.is_empty(), "{:?}", output.stdout);
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    }
}
