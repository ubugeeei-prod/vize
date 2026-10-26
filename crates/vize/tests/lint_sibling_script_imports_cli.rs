#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "native CLI fixture packets"
)]

use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const RULE: &str = "vue/no-unused-setup-bindings";
const SETUP: &str = "import { helper } from './helper';\nconst genuinelyUnused = 0;";
const CASES: &[(&str, bool)] = &[
    (
        "export function describe(value: string) { return helper(value); }",
        true,
    ),
    ("export const describe = () => helper('x');", true),
    ("export const object = { helper };", true),
    ("export const object = { [helper]: true };", true),
    ("export type Helper = typeof helper;", true),
    ("export type Helper = helper;", true),
    ("export const text = 'helper'; // helper('x')", false),
    ("export const object = { helper: true };", false),
    ("export const object = { helper() { return 1; } };", false),
    ("export const describe = () => object.helper();", false),
    ("helper = replacement;", false),
    (
        "export function describe(helper: string) { return helper; }",
        false,
    ),
    (
        "export function describe() { const helper = () => 1; return helper(); }",
        false,
    ),
    ("export type Helper<helper> = { value: helper };", false),
];

fn details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn message(source: &str, setup: &str, name: &str, severity: &str) -> Value {
    let within_setup = setup.find(name).unwrap();
    let start = source.find(setup).unwrap() + within_setup;
    let before = source.get(..start).unwrap();
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    json!({
        "ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md",
        "severity":if severity == "error" {2} else {1},
        "message":format!("[vize:{RULE}] Setup binding '{name}' is never read"),
        "line":line,"column":column,"endLine":line,"endColumn":column+name.encode_utf16().count(),
        "help":"Remove the binding or prefix its name with underscore if intentionally unused"
    })
}

fn matrix(newline: &str, setup_first: bool, severity: &str, profile: bool) {
    let project = tempfile::tempdir().unwrap();
    vite_config::write(
        project.path(),
        json!({"linter":{"preset":"incremental","rules":{RULE:severity}}}).to_string(),
    );
    fs::create_dir(project.path().join("src")).unwrap();
    let mut expected = Vec::new();
    let mut files = Vec::new();
    for (index, &(plain, used)) in CASES.iter().enumerate() {
        let file = format!("src/case-{index:02}.vue");
        let setup = SETUP.replace('\n', newline);
        let normal_block = format!(
            "<script lang=\"ts\">{newline}// 雪🦀 original block{newline}{plain}{newline}</script>"
        );
        let setup_block = format!("<script setup lang=\"ts\">{newline}{setup}{newline}</script>");
        let source = if setup_first {
            format!("{setup_block}{newline}{normal_block}")
        } else {
            format!("{normal_block}{newline}{setup_block}")
        };
        let source = format!("{source}{newline}<template><div /></template>{newline}");
        fs::write(project.path().join(&file), &source).unwrap();
        let mut messages = Vec::new();
        if !used {
            messages.push(message(&source, &setup, "helper", severity));
        }
        messages.push(message(&source, &setup, "genuinelyUnused", severity));
        let count = messages.len();
        expected.push(json!({"file":file,"messages":messages,"errorCount":if severity == "error" {count} else {0},"warningCount":if severity == "warn" {count} else {0}}));
        files.push(file);
    }
    let run = |profiling| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(project.path())
            .args(["lint", "--config", "vite.config.mjs", "--format", "json"])
            .args(&files);
        if profiling {
            command.args(["--profile-json", "profile.json"]);
        }
        command.output().unwrap()
    };
    let plain = run(false);
    assert_eq!(
        plain.status.code(),
        Some(i32::from(severity == "error")),
        "{}",
        details(&plain)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&plain.stdout).unwrap(),
        json!(expected),
        "{}",
        details(&plain)
    );
    if profile {
        let observed = run(true);
        assert_eq!(observed.status.code(), plain.status.code());
        assert_eq!(observed.stdout, plain.stdout);
        let profile: Value =
            serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap())
                .unwrap();
        for (key, count) in [
            ("croquis.script_plain.oxc_parse", CASES.len()),
            ("croquis.script_setup.oxc_parse", 2 * CASES.len()),
            ("croquis.script_plain.module_reads", CASES.len()),
        ] {
            let calls: u64 = profile["spans"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|span| span["key"] == key)
                .map(|span| span["count"].as_u64().unwrap())
                .sum();
            assert_eq!(
                calls, count as u64,
                "{key}: original parse counts and one demanded module analysis"
            );
        }
        assert_eq!(
            profile["truncation"],
            json!({"dropped_spans":0,"dropped_counters":0})
        );
    }
}

#[test]
fn sibling_script_import_reads_preserve_complete_cli_packets_and_existing_parse_counts() {
    for newline in ["\n", "\r\n"] {
        for setup_first in [false, true] {
            for severity in ["warn", "error"] {
                matrix(newline, setup_first, severity, severity == "warn");
            }
        }
    }
}

#[test]
fn published_reproduction_and_negative_control_are_executed_from_pinned_inputs() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/sibling-script-import-reads-10616");
    let project = tempfile::tempdir().unwrap();
    fs::copy(
        inputs.join("vite.config.mjs"),
        project.path().join("vite.config.mjs"),
    )
    .unwrap();
    for (name, expected_names) in [
        ("TwoBlocks.lf.vue", &[][..]),
        ("TwoBlocks.crlf.vue", &[][..]),
        ("Shadowed.vue", &["helper"][..]),
    ] {
        let source = fs::read_to_string(inputs.join(format!("{name}.txt"))).unwrap();
        fs::write(project.path().join(name), &source).unwrap();
        let setup = source
            .split("<script setup lang=\"ts\">")
            .nth(1)
            .unwrap()
            .split("</script>")
            .next()
            .unwrap();
        let messages: Vec<_> = expected_names
            .iter()
            .map(|name| message(&source, setup, name, "warn"))
            .collect();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                name,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", details(&output));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!([{"file":name,"messages":messages,"errorCount":0,"warningCount":expected_names.len()}]),
            "{}",
            details(&output)
        );
    }
}

#[test]
fn disabled_rule_does_not_collect_normal_module_read_facts() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/sibling-script-import-reads-10616");
    let project = tempfile::tempdir().unwrap();
    fs::copy(
        inputs.join("TwoBlocks.lf.vue.txt"),
        project.path().join("TwoBlocks.vue"),
    )
    .unwrap();
    vite_config::write(
        project.path(),
        json!({"linter":{"preset":"incremental","rules":{RULE:"off"}}}).to_string(),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "--profile-json",
            "profile.json",
            "TwoBlocks.vue",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", details(&output));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([{"file":"TwoBlocks.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
    let profile: Value =
        serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap()).unwrap();
    assert!(
        profile["spans"]
            .as_array()
            .unwrap()
            .iter()
            .all(|span| span["key"] != "croquis.script_plain.module_reads")
    );
}
