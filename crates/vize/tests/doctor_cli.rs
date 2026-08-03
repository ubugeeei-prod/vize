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

#[test]
fn malformed_script_modules_cannot_produce_a_healthy_report() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/broken.ts",
        "export const answer: number =",
    );

    let output = doctor(directory.path(), &["src", "--format", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot parse script module"));
    assert!(output.stdout.is_empty());
}

#[test]
fn malformed_sfc_scripts_cannot_produce_a_healthy_report() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/Broken.vue",
        "<script setup lang=\"ts\">const answer: number =</script>\n<template><p /></template>",
    );

    let output = doctor(directory.path(), &["src", "--format", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot parse component script"));
    assert!(output.stdout.is_empty());
}

#[test]
fn valid_script_module_dialects_remain_supported() {
    let directory = tempfile::tempdir().unwrap();
    for (path, source) in [
        ("src/value.js", "export const value = 1"),
        ("src/component.jsx", "export const View = () => <div />"),
        ("src/value.ts", "export const value: number = 1"),
        (
            "src/component.tsx",
            "export const View = () => <div data-value={1} />",
        ),
        ("src/value.mjs", "export const value = 1"),
        ("src/value.cjs", "module.exports = { value: 1 }"),
        ("src/value.mts", "export const value: number = 1"),
        ("src/value.cts", "module.exports = { value: 1 as number }"),
    ] {
        write(directory.path(), path, source);
    }
    write(
        directory.path(),
        "src/TsxComponent.vue",
        "<script setup lang=\"tsx\">const View = () => <div /></script>\n<template><View /></template>",
    );

    let output = doctor(directory.path(), &["src", "--format", "json"]);
    let report: DoctorReport = serde_json::from_slice(&output.stdout).unwrap();

    assert!(output.status.success());
    assert_eq!(report.summary().counts.total(), 0);
}

#[test]
fn public_sfc_contract_is_explicit_and_source_accurate() {
    let directory = tempfile::tempdir().unwrap();
    let source = "<script setup>const label = 'Save'</script>\n\
                  <template><button>{{ label }}</button></template>\n\
                  <style>button { font: inherit }</style>\n";
    write(directory.path(), "src/ActionButton.vue", source);

    let ordinary = doctor(directory.path(), &["src", "--format", "json"]);
    let audited = doctor(
        directory.path(),
        &["src", "--format", "json", "--public-sfc"],
    );
    let ordinary_report: DoctorReport = serde_json::from_slice(&ordinary.stdout).unwrap();
    let audited_report: DoctorReport = serde_json::from_slice(&audited.stdout).unwrap();
    let finding = audited_report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_SFC_EXPLICIT_SECTIONS")
        .unwrap();

    assert!(ordinary.status.success());
    assert!(
        ordinary_report
            .findings()
            .iter()
            .all(|finding| finding.code != "VIZE_DOCTOR_SFC_EXPLICIT_SECTIONS")
    );
    assert_eq!(audited.status.code(), Some(1));
    assert_eq!(finding.primary.path, "src/ActionButton.vue");
    assert_eq!(
        &source[finding.primary.start as usize..finding.primary.end as usize],
        "<script setup>"
    );
    assert_eq!(finding.evidence.len(), 2);
}

#[test]
fn public_sfc_json_is_deterministic_for_input_order() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/A.vue",
        "<template><p>A</p></template>",
    );
    write(
        directory.path(),
        "src/B.vue",
        "<template><p>B</p></template>",
    );

    let first = doctor(
        directory.path(),
        &["src/A.vue", "src/B.vue", "--format", "json", "--public-sfc"],
    );
    let second = doctor(
        directory.path(),
        &["src/B.vue", "src/A.vue", "--format", "json", "--public-sfc"],
    );

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
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
