//! Issue #7935: project findings honor per-file rule and category configuration.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "CLI fixture uses std strings")]

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const ROUTE_CODES: [&str; 4] = [
    "ecosystem/vue-router-unknown-route",
    "ecosystem/vue-router-extra-param",
    "ecosystem/vue-router-param-type",
    "ecosystem/vue-router-missing-param",
];

fn write(root: &Path, path: &str, source: &str) {
    fs::write(root.join(path), source).expect("write fixture");
}

fn run(config: Value, source: &str, name: &str) -> (Value, i32) {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), name, source);
    write(
        dir.path(),
        "vize.config.json",
        &serde_json::to_string(&config).expect("config JSON"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir.path())
        .args([
            "lint",
            "--cross-file",
            "--format",
            "json",
            "--help-level",
            "none",
            name,
        ])
        .output()
        .expect("run lint");
    assert_eq!(output.stderr, b"", "stderr must remain empty");
    (
        serde_json::from_slice(&output.stdout).expect("JSON report"),
        output.status.code().expect("exit code"),
    )
}

fn expected_routes() -> Value {
    serde_json::from_str(include_str!("fixtures/issue-7935/routes.expected.json"))
        .expect("route reference")
}

fn expected_consumer() -> Value {
    serde_json::from_str(include_str!("fixtures/issue-7935/Consumer.expected.json"))
        .expect("inject reference")
}

fn configure_expected(mut report: Value, severity: &str) -> Value {
    for file in report.as_array_mut().unwrap() {
        let rows = file["messages"].as_array_mut().unwrap();
        if severity == "off" {
            rows.clear();
        } else {
            for message in rows.iter_mut() {
                message["severity"] = json!(if severity == "warn" { 1 } else { 2 });
            }
        }
        let count = rows.len();
        file["errorCount"] = json!(if severity == "error" { count } else { 0 });
        file["warningCount"] = json!(if severity == "warn" { count } else { 0 });
    }
    report
}

#[test]
fn all_four_route_rules_accept_off_warn_and_error_without_changing_ranges() {
    let source = include_str!("fixtures/issue-7935/routes.ts.fixture");
    let (baseline, status) = run(
        json!({"linter":{"preset":"incremental"}}),
        source,
        "routes.ts",
    );
    assert_eq!(baseline, expected_routes());
    assert_eq!(status, 1);
    for severity in ["off", "warn", "error"] {
        let rules: serde_json::Map<_, _> = ROUTE_CODES
            .iter()
            .map(|code| ((*code).to_owned(), json!(severity)))
            .collect();
        let (actual, status) = run(
            json!({"linter":{"preset":"incremental","rules":rules}}),
            source,
            "routes.ts",
        );
        assert_eq!(
            actual,
            configure_expected(expected_routes(), severity),
            "{severity} preserves every other JSON field"
        );
        assert_eq!(status, if severity == "error" { 1 } else { 0 });
    }
}

#[test]
fn cross_file_group_and_individual_codes_control_inject_findings() {
    let source = include_str!("fixtures/issue-7935/Consumer.vue.fixture");
    let (baseline, status) = run(
        json!({"linter":{"preset":"incremental"}}),
        source,
        "Consumer.vue",
    );
    assert_eq!(baseline, expected_consumer());
    assert_eq!(status, 1);
    for settings in [
        json!({"rules":{"cross-file":"off"}}),
        json!({"categories":{"cross-file":"off"}}),
    ] {
        let mut config = settings;
        config["preset"] = json!("incremental");
        let (report, status) = run(json!({"linter":config}), source, "Consumer.vue");
        assert_eq!(report, configure_expected(expected_consumer(), "off"));
        assert_eq!(status, 0);
    }
    for settings in [
        json!({"rules":{"cross-file":"warn"}}),
        json!({"categories":{"cross-file":"warn"}}),
        json!({"rules":{"vize:croquis/cf/unmatched-inject":"warn"}}),
    ] {
        let mut config = settings;
        config["preset"] = json!("incremental");
        let (report, status) = run(json!({"linter":config}), source, "Consumer.vue");
        assert_eq!(report, configure_expected(expected_consumer(), "warn"));
        assert_eq!(status, 0);
    }
    let (report, status) = run(
        json!({"linter":{"preset":"incremental","rules":{
            "vize:croquis/cf/unmatched-inject":"off"
        }}}),
        source,
        "Consumer.vue",
    );
    let mut expected = expected_consumer();
    expected[0]["messages"].as_array_mut().unwrap().remove(0);
    expected[0]["errorCount"] = json!(0);
    assert_eq!(report, expected);
    assert_eq!(status, 0);
}

#[test]
fn entries_and_category_precedence_apply_to_the_file_with_the_finding() {
    let source = include_str!("fixtures/issue-7935/routes.ts.fixture");
    let rules: serde_json::Map<_, _> = ROUTE_CODES
        .iter()
        .map(|code| ((*code).to_owned(), json!("off")))
        .collect();
    let (report, status) = run(
        json!({
            "linter":{"preset":"incremental"},
            "entries":[{"files":["routes.ts"],"linter":{"rules":rules}}]
        }),
        source,
        "routes.ts",
    );
    assert_eq!(report, configure_expected(expected_routes(), "off"));
    assert_eq!(status, 0);
    for category in ["ecosystem", "suspicious"] {
        let (report, status) = run(
            json!({"linter":{
                "preset":"incremental","categories":{category:"off"},
                "rules":{"ecosystem/vue-router-unknown-route":"error"}
            }}),
            source,
            "routes.ts",
        );
        assert_eq!(report, configure_expected(expected_routes(), "off"));
        assert_eq!(status, 0);
    }
    let (report, status) = run(
        json!({"linter":{
            "preset":"incremental","categories":{"ecosystem":"warn"},
            "rules":{"ecosystem/vue-router-unknown-route":"error"}
        }}),
        source,
        "routes.ts",
    );
    let mut expected = configure_expected(expected_routes(), "warn");
    expected[0]["messages"][0]["severity"] = json!(2);
    expected[0]["errorCount"] = json!(1);
    expected[0]["warningCount"] = json!(3);
    assert_eq!(report, expected);
    assert_eq!(status, 1);
}
