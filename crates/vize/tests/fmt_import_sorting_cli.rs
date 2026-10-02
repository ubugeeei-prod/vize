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

#[test]
fn malformed_sort_settings_fail_closed_and_no_config_bypasses_them() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    fs::write(root.join("UserCard.vue"), SOURCE).expect("source");
    for invalid in [
        r#"{"formatter":{"sortImports":{"groups":{}}}}"#,
        r#"{"formatter":{"sortImports":{"newlineBetween":false}}}"#,
        r#"{"formatter":{"sortImports":{"customGroups":[{"groupName":"local","selectorTypo":"type"}]}}}"#,
        r#"{"formatter":{"sortImports":{"groups":[{"newlinesBetween":true},"external"]}}}"#,
        r#"{"formatter":{"sortImports":{"groups":["external",{"newlinesBetween":true}]}}}"#,
        r#"{"formatter":{"sortImports":{"groups":["external",{"newlinesBetween":true},{"newlinesBetween":false},"sibling"]}}}"#,
        "{",
    ] {
        fs::write(root.join("vize.config.json"), invalid).expect("invalid config");
        assert_eq!(
            fmt(root, &["--write", "UserCard.vue"]).status.code(),
            Some(2)
        );
        assert_eq!(
            fs::read_to_string(root.join("UserCard.vue")).expect("protected source"),
            SOURCE
        );
        assert_eq!(
            fmt(root, &["--no-config", "--check", "UserCard.vue"])
                .status
                .code(),
            Some(0)
        );
    }
}

#[test]
fn sorting_entries_and_ignores_share_one_effectful_js_config_evaluation() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    fs::create_dir(root.join("nested")).expect("scope");
    for name in ["UserCard.vue", "Ignored.vue"] {
        fs::write(root.join("nested").join(name), SOURCE).expect("source");
    }
    fs::write(root.join("eval-count.txt"), "0").expect("counter");
    fs::write(
        root.join("vize.config.mjs"),
        r#"
import { readFileSync, writeFileSync } from 'node:fs';
const counter = new URL('./eval-count.txt', import.meta.url);
export default () => {
  const count = Number(readFileSync(counter, 'utf8')) + 1;
  writeFileSync(counter, String(count));
  return { formatter: { sortImports: count === 1 ? {} : false },
    entries: [{ basePath: 'nested', files: ['*.vue'], ignores: ['Ignored.vue'] }] };
};
"#,
    )
    .expect("effectful config");
    assert_eq!(
        fmt(root, &["--config", "vize.config.mjs", "--write", "*.vue"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        fs::read_to_string(root.join("nested/UserCard.vue")).expect("sorted source"),
        EXPECTED
    );
    assert_eq!(
        fs::read_to_string(root.join("nested/Ignored.vue")).expect("ignored source"),
        SOURCE
    );
    assert_eq!(
        fs::read_to_string(root.join("eval-count.txt")).expect("counter"),
        "1"
    );
}
