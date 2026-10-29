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
                "json",
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
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            serde_json::json!([{
                "file": file.to_str().unwrap(), "messages": [],
                "errorCount": 0, "warningCount": 0,
            }])
        );
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    }
}

#[test]
fn lint_fix_prefers_the_define_emits_binding_and_preserves_report_only_cases() {
    const RULE: &str = "script/prefer-define-emits-binding";
    const SOURCE: &str = r#"<script setup lang="ts">
const dispatch = defineEmits<(event: string, payload: unknown) => void>()
</script>

<template>
  <button aria-label="閉じる" @click="$emit(eventName, payload)">Close</button>
</template>
"#;
    const FIXED: &str = r#"<script setup lang="ts">
const dispatch = defineEmits<(event: string, payload: unknown) => void>()
</script>

<template>
  <button aria-label="閉じる" @click="dispatch(eventName, payload)">Close</button>
</template>
"#;
    const CONFIG: &str = r#"{
  "vue": {
    "linter": {
      "preset": "incremental",
      "rules": { "script/prefer-define-emits-binding": "warn" }
    }
  }
}"#;

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("App.vue");
    fs::write(&file, SOURCE).unwrap();
    fs::write(dir.path().join("config.json"), CONFIG).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(dir.path())
            .args(["lint", "--config", "config.json", "--format", "json"])
            .args(extra)
            .arg("App.vue")
            .output()
            .unwrap()
    };

    let report = run(&[]);
    assert_eq!(report.status.code(), Some(0));
    assert!(report.stderr.is_empty(), "{:?}", report.stderr);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&report.stdout).unwrap(),
        serde_json::json!([{
            "file": "App.vue",
            "messages": [{
                "ruleId": RULE,
                "ruleDocsPath": "docs/content/rules/type-and-script.md",
                "severity": 1,
                "message": "[vize:script/prefer-define-emits-binding] Use the function returned by defineEmits() instead of template `$emit`.",
                "line": 6,
                "column": 36,
                "endLine": 6,
                "endColumn": 41,
                "help": "Call dispatch(...) so event names and payloads are checked against the defineEmits contract."
            }],
            "errorCount": 0,
            "warningCount": 1
        }])
    );

    for _ in 0..2 {
        let fixed = run(&["--fix"]);
        assert_eq!(fixed.status.code(), Some(0));
        assert!(fixed.stderr.is_empty(), "{:?}", fixed.stderr);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fixed.stdout).unwrap(),
            serde_json::json!([{
                "file": "App.vue", "messages": [], "errorCount": 0, "warningCount": 0
            }])
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), FIXED);
    }

    let report_only = r#"<script setup lang="ts">
defineEmits<(event: string) => void>()
</script>
<template><button @click="$emit(eventName)">Close</button></template>
"#;
    fs::write(&file, report_only).unwrap();
    let unchanged = run(&["--fix"]);
    assert_eq!(unchanged.status.code(), Some(0));
    assert!(unchanged.stderr.is_empty(), "{:?}", unchanged.stderr);
    let report: serde_json::Value = serde_json::from_slice(&unchanged.stdout).unwrap();
    assert_eq!(report[0]["warningCount"], serde_json::json!(1));
    assert_eq!(report[0]["messages"][0]["ruleId"], serde_json::json!(RULE));
    assert_eq!(
        report[0]["messages"][0]["help"],
        serde_json::json!(
            "Capture the only defineEmits() call in an immutable plain identifier before replacing $emit."
        )
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), report_only);
}
