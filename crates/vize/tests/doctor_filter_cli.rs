use std::{fs, path::Path, process::Command};

use vize_doctor::DoctorReport;

#[test]
fn multidimensional_filters_share_glob_and_evidence_graph_semantics() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/A.vue",
        "<template><input id=\"shared\" /></template>",
    );
    write(
        directory.path(),
        "src/B.vue",
        "<template><input id=\"shared\" /></template>",
    );

    let selected = doctor(
        directory.path(),
        &[
            "src",
            "--format",
            "json",
            "--category",
            "accessibility,correctness",
            "--severity",
            "warning",
            "--confidence",
            "certain",
            "--rule",
            "VIZE_DOCTOR_CF_DUPLICATE_*",
            "--path",
            "src/*.vue",
            "--changed-file",
            "src/B.vue",
        ],
    );
    let report: DoctorReport = serde_json::from_slice(&selected.stdout).unwrap();

    assert!(selected.status.success());
    assert_eq!(report.findings().len(), 1);
    assert_eq!(report.findings()[0].code, "VIZE_DOCTOR_CF_DUPLICATE_ID");
    assert_eq!(report.findings()[0].primary.path, "src/A.vue");
    assert_eq!(report.findings()[0].related[0].location.path, "src/B.vue");
}

#[test]
fn filters_recompute_health_and_exit_from_visible_findings() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/Parent.vue",
        r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 0 })
</script>
<template><Child :item="state" /></template>"#,
    );
    write(
        directory.path(),
        "src/Child.vue",
        r#"<script setup lang="ts">
const props = defineProps<{ item: { count: number } }>()
const { item } = props
</script>"#,
    );

    let unfiltered = doctor(directory.path(), &["src", "--format", "json"]);
    let hidden = doctor(
        directory.path(),
        &["src", "--format", "json", "--target", "native"],
    );
    let hidden_report: DoctorReport = serde_json::from_slice(&hidden.stdout).unwrap();

    assert_eq!(unfiltered.status.code(), Some(1));
    assert!(hidden.status.success());
    assert!(hidden_report.findings().is_empty());
    assert_eq!(hidden_report.summary().overall_score, 100);
    assert!(!hidden_report.summary().has_blocking_errors);
}

#[test]
fn malformed_filter_globs_fail_before_emitting_a_report() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/App.vue",
        "<template><main /></template>",
    );

    let output = doctor(
        directory.path(),
        &["src", "--format", "json", "--changed-file", "[broken"],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid changed-file filter pattern")
    );
}

fn doctor(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("doctor")
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
