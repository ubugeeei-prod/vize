//! Configured cross-file checks and warning limits match their CLI counterparts.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "subprocess fixture reports")]
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp project");
    for (name, source) in [
        ("App.vue", include_str!("fixtures/lint-execution/App.vue")),
        (
            "Child.vue",
            include_str!("fixtures/lint-execution/Child.vue"),
        ),
        (
            "Warning.vue",
            include_str!("fixtures/lint-execution/Warning.vue"),
        ),
    ] {
        fs::write(dir.path().join(name), source).expect("fixture");
    }
    dir
}

fn lint(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--format", "json"])
        .args(args)
        .output()
        .expect("lint")
}

fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("complete JSON report")
}

fn config(root: &Path, linter: Value) {
    fs::write(
        root.join("vize.config.json"),
        serde_json::to_vec(&json!({"linter": linter})).expect("config JSON"),
    )
    .expect("write config");
}

#[test]
fn config_cross_file_checks_match_cli_and_no_config_disables_them() {
    let project = project();
    let root = project.path();
    let flags = lint(
        root,
        &["--no-config", "--cross-file", "App.vue", "Child.vue"],
    );
    assert_eq!(flags.status.code(), Some(1));
    let baseline = lint(root, &["--no-config", "App.vue", "Child.vue"]);
    assert_ne!(report(&flags), report(&baseline));
    for (switch, flag) in [
        ("crossFile", "--cross-file"),
        ("crossFileTree", "--cross-file-tree"),
        ("crossFileComplexity", "--cross-file-complexity"),
    ] {
        let flags = lint(root, &["--no-config", flag, "App.vue", "Child.vue"]);
        config(root, json!({(switch): true}));
        let configured = lint(root, &["App.vue", "Child.vue"]);
        assert_eq!(configured.status.code(), flags.status.code());
        assert_eq!(report(&configured), report(&flags));
        assert_eq!(
            report(&lint(root, &["--no-config", "App.vue", "Child.vue"])),
            report(&baseline)
        );
    }
}

#[test]
fn warning_limit_config_fails_and_cli_can_override_it() {
    let project = project();
    let root = project.path();
    config(
        root,
        json!({"maxWarnings": 0, "rules": {"a11y/alt-text": "warn"}}),
    );
    let configured = lint(root, &["Warning.vue"]);
    let cli_zero = lint(root, &["--max-warnings", "0", "Warning.vue"]);
    let unlimited = lint(root, &["--max-warnings", "100", "Warning.vue"]);
    assert_eq!(configured.status.code(), Some(1));
    assert_eq!(cli_zero.status.code(), Some(1));
    assert_eq!(unlimited.status.code(), Some(0));
    assert_eq!(report(&configured), report(&cli_zero));
    assert_eq!(report(&configured), report(&unlimited));
    assert_eq!(report(&configured)[0]["warningCount"], json!(1));
    assert_eq!(report(&configured)[0]["errorCount"], json!(0));
}
