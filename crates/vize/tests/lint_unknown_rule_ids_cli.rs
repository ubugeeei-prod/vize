//! Complete CLI configuration contracts for the original unknown-rule report.
#![cfg(test)]

use serde_json::{Value, json};
use std::{error::Error, fs, path::Path, process::Command};
use vize_l0::cstr;

type TestResult = Result<(), Box<dyn Error>>;
const SOURCE: &[u8] =
    include_bytes!("../../../tests/_fixtures/differential/linter/unknown-rule-ids/MyText.vue.txt");
const CONFIG: &[u8] = include_bytes!(
    "../../../tests/_fixtures/differential/linter/unknown-rule-ids/vize.config.json"
);
const CORPUS: &[u8] =
    include_bytes!("../../../tests/_fixtures/differential/linter/unknown-rule-ids/corpus.json");

fn write(root: &Path, path: &str, bytes: &[u8]) -> TestResult {
    let file = root.join(path);
    let parent = file.parent().ok_or("fixture file needs a parent")?;
    fs::create_dir_all(parent)?;
    fs::write(file, bytes)?;
    Ok(())
}

fn run(root: &Path, args: &[&str]) -> Result<std::process::Output, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()?)
}

fn error(root: &Path, args: &[&str], scope: &str, ids: &str) -> TestResult {
    let output = run(root, args)?;
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(
        output.stderr,
        cstr!("\x1b[31mError:\x1b[0m Unknown lint rule IDs in configuration:\n  {scope}: {ids}\n")
            .as_bytes()
    );
    Ok(())
}

fn json_result(root: &Path, args: &[&str], expected: &Value) -> TestResult {
    let output = run(root, args)?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(serde_json::from_slice::<Value>(&output.stdout)?, *expected);
    Ok(())
}

#[test]
fn original_report_names_all_unknown_ids_before_lint_or_fix() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    write(root.path(), "vize.config.json", CONFIG)?;
    let corpus = serde_json::from_slice::<Value>(CORPUS)?;
    let expected = corpus
        .get("originalExpected")
        .ok_or("missing original contract")?;
    for args in [
        vec!["lint", "-f", "plain", "--help-level", "none", "MyText.vue"],
        vec!["lint", "--fix", "--format", "json", "MyText.vue"],
        vec!["lint", "--format", "json", "absent/**/*.vue"],
    ] {
        let output = run(root.path(), &args)?;
        assert_eq!(
            output.status.code(),
            expected
                .get("status")
                .and_then(Value::as_i64)
                .map(|v| v as i32)
        );
        assert_eq!(
            output.stdout,
            expected
                .get("stdout")
                .and_then(Value::as_str)
                .ok_or("missing stdout")?
                .as_bytes()
        );
        assert_eq!(
            output.stderr,
            expected
                .get("stderr")
                .and_then(Value::as_str)
                .ok_or("missing stderr")?
                .as_bytes()
        );
        assert_eq!(fs::read(root.path().join("MyText.vue"))?, SOURCE);
    }
    Ok(())
}

#[test]
fn unknown_root_and_unmatched_custom_entry_ids_reject_every_severity() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    for severity in ["off", "warn", "error"] {
        let config = json!({"linter": {"preset": "incremental", "rules": {"vue/no-inline-styles": severity}}});
        write(
            root.path(),
            "vize.config.json",
            &serde_json::to_vec(&config)?,
        )?;
        error(
            root.path(),
            &["lint", "MyText.vue"],
            "linter.rules",
            "vue/no-inline-styles",
        )?;

        let config = json!({
            "linter": {"preset": "incremental"},
            "entries": [{"basePath": "packages/admin", "files": ["**/*.vue"],
                "linter": {"rules": {"a11y/no-autofocuss": severity}}}]
        });
        write(
            root.path(),
            "vize.config.json",
            &serde_json::to_vec(&config)?,
        )?;
        error(
            root.path(),
            &["lint", "MyText.vue"],
            "entries[].linter.rules",
            "a11y/no-autofocuss",
        )?;
    }
    Ok(())
}

#[test]
fn overwritten_entry_ids_are_still_validated_in_sorted_declaration_scope() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    let config = json!({
        "linter": {"preset": "incremental"},
        "entries": [
            {"files": ["**/*.vue"], "linter": {"rules": {"z/not-a-rule": "warn", "a/not-a-rule": "error"}}},
            {"files": ["**/*.vue"], "linter": {"rules": {"z/not-a-rule": "off"}}}
        ]
    });
    write(
        root.path(),
        "vize.config.json",
        &serde_json::to_vec(&config)?,
    )?;
    error(
        root.path(),
        &["lint", "MyText.vue"],
        "entries[].linter.rules",
        "a/not-a-rule, z/not-a-rule",
    )
}

#[test]
fn no_config_ignores_the_invalid_document_without_changing_default_execution() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    write(root.path(), "vize.config.json", CONFIG)?;
    let corpus = serde_json::from_slice::<Value>(CORPUS)?;
    json_result(
        root.path(),
        &[
            "lint",
            "--no-config",
            "--preset",
            "incremental",
            "--format",
            "json",
            "MyText.vue",
        ],
        corpus
            .get("cleanExpected")
            .ok_or("missing clean contract")?,
    )
}

#[test]
fn actual_registered_families_and_generated_category_markers_remain_valid() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    let corpus = serde_json::from_slice::<Value>(CORPUS)?;
    let names = corpus
        .get("knownControlIds")
        .and_then(Value::as_array)
        .ok_or("missing registered controls")?;
    let rules = names
        .iter()
        .map(|name| {
            Ok((
                name.as_str().ok_or("rule ID must be a string")?.to_owned(),
                json!("off"),
            ))
        })
        .collect::<Result<serde_json::Map<_, _>, Box<dyn Error>>>()?;
    for preset in ["incremental", "essential", "nuxt"] {
        let config = json!({
            "linter": {"preset": preset, "categories": {"style": "off"}, "rules": rules},
            "entries": [
                {"files": ["**/*.vue"], "linter": {"rules": rules}},
                {"files": ["never/**/*.vue"], "linter": {"typeAware": true, "categories": {"style": "off"}}}
            ]
        });
        write(
            root.path(),
            "vize.config.json",
            &serde_json::to_vec(&config)?,
        )?;
        // Override only the execution preset, keeping recognition independent of it.
        json_result(
            root.path(),
            &[
                "lint",
                "--preset",
                "incremental",
                "--format",
                "json",
                "MyText.vue",
            ],
            corpus
                .get("cleanExpected")
                .ok_or("missing clean contract")?,
        )?;
    }
    Ok(())
}

#[test]
fn corrected_root_and_custom_entry_rules_preserve_the_complete_warning() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "MyText.vue", SOURCE)?;
    write(root.path(), "packages/admin/MyText.vue", SOURCE)?;
    let corpus = serde_json::from_slice::<Value>(CORPUS)?;
    let expected = corpus
        .get("warningExpected")
        .ok_or("missing warning contract")?;
    for config in [
        json!({"linter": {"preset": "incremental", "rules": {"vue/no-inline-style": "warn"}}}),
        json!({"linter": {"preset": "incremental"}, "entries": [{"basePath": "packages/admin", "files": ["**/*.vue"], "linter": {"rules": {"vue/no-inline-style": "warn"}}}]}),
    ] {
        write(
            root.path(),
            "vize.config.json",
            &serde_json::to_vec(&config)?,
        )?;
        let output = run(
            root.path(),
            &[
                "lint",
                "--format",
                "json",
                "--help-level",
                "none",
                "packages/admin/MyText.vue",
            ],
        )?;
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stderr, Vec::<u8>::new());
        let mut wanted = expected.clone();
        let record = wanted
            .as_array_mut()
            .and_then(|rows| rows.first_mut())
            .ok_or("missing warning row")?;
        record
            .as_object_mut()
            .ok_or("warning row needs fields")?
            .insert("file".into(), json!("packages/admin/MyText.vue"));
        assert_eq!(serde_json::from_slice::<Value>(&output.stdout)?, wanted);
    }
    Ok(())
}
