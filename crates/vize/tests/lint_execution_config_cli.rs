//! Configured cross-file checks and warning limits match their CLI counterparts.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "subprocess fixture reports")]
#[path = "support/config_refusal.rs"]
mod config_refusal;
#[path = "support/vite_config.rs"]
mod vite_config;
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
    lint_command(root, args).output().expect("lint")
}

fn lint_command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .args(["lint", "--format", "json"])
        .args(args);
    command
}

fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("complete JSON report")
}

fn config(root: &Path, linter: Value) {
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(serde_json::to_vec(&json!({"linter": linter})).expect("config JSON")),
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

#[test]
fn invalid_explicit_json_settings_fail_before_fix_and_no_config_can_ignore_them() {
    let project = project();
    let root = project.path();
    let before = fs::read(root.join("Warning.vue")).expect("source");
    for invalid in [r#"{"linter":{"maxWarnings":-1}}"#, "{"] {
        fs::write(root.join("config.json"), invalid).expect("invalid config");
        let error = serde_json::from_str::<vize_l0::config::ConfigDocument>(invalid)
            .expect_err("invalid typed config");
        for args in [
            vec!["--config", "config.json", "--fix", "Warning.vue"],
            vec!["--config", "config.json", "Warning.vue"],
        ] {
            let output = lint(root, &args);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            let expected =
                vize_l0::cstr!("\x1b[31mError:\x1b[0m failed to parse config.json: {error}\n");
            assert_eq!(String::from_utf8_lossy(&output.stderr), expected.as_str());
            assert_eq!(
                fs::read(root.join("Warning.vue")).expect("unchanged source"),
                before
            );
        }
        assert_ne!(
            lint(root, &["--no-config", "Warning.vue"]).status.code(),
            Some(2)
        );
    }
}

#[test]
fn invalid_discovered_vite_settings_fail_before_fix_and_no_config_can_ignore_them() {
    let project = project();
    let root = project.path();
    let before = fs::read(root.join("Warning.vue")).expect("source");
    for (invalid, exception) in [
        (
            "export default { vue: { linter: { maxWarnings: -1 } } };",
            "TypeError: vue.linter.maxWarnings: must be >= 0",
        ),
        (
            "export default { vue: { };",
            "SyntaxError: Unexpected token ';'",
        ),
    ] {
        fs::write(root.join("vite.config.mjs"), invalid).expect("invalid Vite config");
        let discovered = root
            .canonicalize()
            .expect("project root")
            .join("vite.config.mjs");
        for (args, path) in [
            (vec!["--fix", "Warning.vue"], discovered.as_path()),
            (
                vec!["--config", "vite.config.mjs", "Warning.vue"],
                Path::new("vite.config.mjs"),
            ),
        ] {
            let output = lint_command(root, &args)
                .env("NODE_OPTIONS", config_refusal::NODE_OPTIONS)
                .output()
                .expect("controlled invalid config refusal");
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            config_refusal::assert_error(&output, path, discovered.parent().unwrap(), exception, 2);
            assert_eq!(
                fs::read(root.join("Warning.vue")).expect("unchanged source"),
                before
            );
        }
        assert_ne!(
            lint(root, &["--no-config", "Warning.vue"]).status.code(),
            Some(2)
        );
    }
}

#[test]
fn explicit_javascript_config_is_evaluated_once() {
    let project = project();
    let root = project.path();
    fs::write(root.join("eval-count.txt"), "0").expect("counter");
    fs::write(
        root.join("vite.config.mjs"),
        r#"
import { readFileSync, writeFileSync } from 'node:fs';
const counter = new URL('./eval-count.txt', import.meta.url);
export default () => {
  const count = Number(readFileSync(counter, 'utf8')) + 1;
  writeFileSync(counter, String(count));
  return { vue: { linter: { maxWarnings: count === 1 ? 0 : 100, rules: { 'a11y/alt-text': 'warn' } } } };
};
"#,
    )
    .expect("JS config");
    let output = lint(root, &["--config", "vite.config.mjs", "Warning.vue"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(report(&output)[0]["warningCount"], json!(1));
    assert_eq!(
        fs::read_to_string(root.join("eval-count.txt")).expect("counter"),
        "1"
    );
}
