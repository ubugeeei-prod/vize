use std::{fs, path::Path, process::Command};
use vize_doctor::DoctorReport;

#[test]
fn json_output_is_versioned_and_uses_workspace_relative_paths() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/A.vue",
        "<template><input id=\"email\" /></template>",
    );
    write(
        directory.path(),
        "src/B.vue",
        "<template><input id=\"email\" /></template>",
    );

    let output = doctor(directory.path(), &["src", "--format", "json"]);
    let report: DoctorReport = serde_json::from_slice(&output.stdout).unwrap();

    assert!(output.status.success());
    assert_eq!(report.format_version(), 1);
    assert_eq!(report.workspace(), ".");
    assert_eq!(report.findings()[0].primary.path, "src/A.vue");
    assert_eq!(report.summary().counts.warnings, 1);
}

#[test]
fn proven_errors_fail_unless_exit_zero_is_requested() {
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

    let blocked = doctor(directory.path(), &["src", "--format", "json"]);
    let allowed = doctor(
        directory.path(),
        &["src", "--format", "json", "--exit-zero"],
    );
    let report: DoctorReport = serde_json::from_slice(&blocked.stdout).unwrap();

    assert_eq!(blocked.status.code(), Some(1));
    assert!(report.summary().has_blocking_errors);
    assert!(allowed.status.success());
}

#[test]
fn inputs_outside_the_workspace_fail_closed() {
    let workspace = tempfile::tempdir().unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let outside_path = outside.path().to_string_lossy();

    let output = doctor(workspace.path(), &[outside_path.as_ref()]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("outside the workspace root"));
}

#[test]
fn malformed_components_cannot_produce_a_healthy_report() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/Broken.vue",
        "<template><section><span></template>",
    );

    let output = doctor(directory.path(), &["src", "--format", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot parse component"));
    assert!(output.stdout.is_empty());
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
