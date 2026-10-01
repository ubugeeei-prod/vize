//! Authored SFC and standalone import blocks use the same configured sort policy.
#![cfg(test)]
use std::{fs, process::Command};

const SOURCE: &str = include_str!("../../vize_glyph/tests/fixtures/sort-imports/UserCard.vue");
const EXPECTED: &str =
    include_str!("../../vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue");

fn fmt(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .arg("fmt")
        .args(args)
        .output()
        .expect("fmt command")
}

#[test]
fn configured_sfc_sorting_is_applied_by_write_and_detected_by_check() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    fs::write(root.join("UserCard.vue"), SOURCE).expect("source");
    fs::write(
        root.join("vize.config.json"),
        r#"{"formatter":{"sortImports":{}}}"#,
    )
    .expect("config");
    assert_eq!(
        fmt(root, &["--no-config", "--check", "UserCard.vue"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        fmt(root, &["--check", "UserCard.vue"]).status.code(),
        Some(1)
    );
    assert_eq!(
        fs::read_to_string(root.join("UserCard.vue")).expect("unmodified source"),
        SOURCE
    );
    assert_eq!(
        fmt(root, &["--write", "UserCard.vue"]).status.code(),
        Some(0)
    );
    assert_eq!(
        fs::read_to_string(root.join("UserCard.vue")).expect("formatted source"),
        EXPECTED
    );
    assert_eq!(
        fmt(root, &["--check", "UserCard.vue"]).status.code(),
        Some(0)
    );
    fs::write(root.join("UserCard.vue"), SOURCE).expect("source");
    fs::write(
        root.join("vize.config.json"),
        r#"{"formatter":{"sortImports":false}}"#,
    )
    .expect("disabled");
    assert_eq!(
        fmt(root, &["--write", "UserCard.vue"]).status.code(),
        Some(0)
    );
    assert_eq!(
        fs::read_to_string(root.join("UserCard.vue")).expect("unchanged source"),
        SOURCE
    );
}

#[test]
fn standalone_script_receives_the_same_native_groups() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    let script = |sfc: &str| {
        sfc.split_once('\n')
            .expect("opening tag")
            .1
            .split_once("</script>")
            .expect("script")
            .0
            .to_owned()
    };
    fs::write(root.join("imports.ts"), script(SOURCE)).expect("script source");
    fs::write(
        root.join("vize.config.json"),
        r#"{"formatter":{"sortImports":{}}}"#,
    )
    .expect("config");
    assert_eq!(fmt(root, &["--write", "imports.ts"]).status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(root.join("imports.ts")).expect("formatted source"),
        script(EXPECTED)
    );
}

#[test]
fn contradictory_sort_settings_cannot_write_authored_sources() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    fs::write(root.join("UserCard.vue"), SOURCE).expect("source");
    fs::write(
        root.join("vize.config.json"),
        r#"{"formatter":{"sortImports":{"partitionByNewline":true}}}"#,
    )
    .expect("invalid config");
    assert_eq!(
        fmt(root, &["--write", "UserCard.vue"]).status.code(),
        Some(2)
    );
    assert_eq!(
        fs::read_to_string(root.join("UserCard.vue")).expect("protected source"),
        SOURCE
    );
}
