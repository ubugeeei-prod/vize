#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command};
use vize_doctor::DoctorReport;

const ACCEPTED: &str = r#"<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
const { open, items } = defineProps<{ open: boolean; items: string[] }>()
const dates = ref<string[]>([])
const step = ref(0)
const percent = ref(0)
const overflowing = ref(false)
const width = ref(0)
const box = useTemplateRef<HTMLElement>('box')
async function fetchDates() { return ['2026-01-01'] }
async function prefetch() {}
let latest = 0
watch(() => open, () => {
  if (!open) return
  const id = ++latest
  fetchDates().then(found => {
    if (id !== latest) return
    dates.value = found
  })
})
watch(() => open, async () => {
  if (!open) return
  step.value = 0
  await prefetch()
})
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  clearInterval(timer)
  percent.value = 0
  timer = setInterval(() => {
    percent.value = Math.min(100, percent.value + 2)
  }, 100)
}
onBeforeUnmount(() => clearInterval(timer))
watch(() => items, () => {
  requestAnimationFrame(() => {
    overflowing.value = (box.value?.scrollWidth ?? 0) > (box.value?.clientWidth ?? 0)
  })
})
watch(() => items, async () => {
  await nextTick()
  width.value = box.value?.offsetWidth ?? 0
})
defineExpose({ start })
</script>
<template>
  <div ref="box" :data-step="step" :data-percent="percent" :data-overflowing="overflowing" :data-width="width">
    {{ dates.join(',') }} {{ items.length }}
  </div>
</template>
"#;

const UNSAFE: &str = r#"<script setup lang="ts">
import { nextTick, ref, useTemplateRef, watch } from 'vue'
const source = ref(0)
const dates_no_guard = ref<string[]>([])
const dates_wrong_guard = ref<string[]>([])
const result = ref<string[]>([])
const percent = ref(0)
const overflowing = ref(false)
const width = ref(0)
const box = useTemplateRef<HTMLElement>('box')
async function fetchDates() { return ['2026-01-01'] }
let latest = 0
watch(source, () => {
  const id = ++latest
  fetchDates().then(found => {
    dates_no_guard.value = found
  })
  fetchDates().then(found => {
    if (id === latest) return
    dates_wrong_guard.value = found
  })
})
watch(source, async () => {
  const found = await fetchDates()
  result.value = found
})
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  timer = setInterval(() => {
    percent.value = Math.min(100, percent.value + 2)
  }, 100)
}
watch(source, () => {
  const measured = box.value?.scrollWidth ?? 0
  requestAnimationFrame(() => {
    overflowing.value = measured > 0
  })
})
watch(source, async () => {
  const measured = box.value?.offsetWidth ?? 0
  await nextTick()
  width.value = measured
})
defineExpose({ start })
</script>
<template>
  <div ref="box" :data-source="source" :data-percent="percent" :data-overflowing="overflowing" :data-width="width">
    {{ dates_no_guard.join(',') }} {{ dates_wrong_guard.join(',') }} {{ result.join(',') }}
  </div>
</template>
"#;

#[test]
fn accepted_boundaries_are_clean_in_doctor_and_cross_file_lint() {
    let directory = fixture();
    let doctor = doctor(directory.path());
    let lint = lint(directory.path());

    assert!(doctor_async_findings(&doctor, "src/Accepted.vue").is_empty());
    assert!(lint_async_findings(&lint, "src/Accepted.vue").is_empty());
}

#[test]
fn unsafe_controls_are_preserved_in_doctor_and_cross_file_lint() {
    let directory = fixture();
    let doctor = doctor(directory.path());
    let lint = lint(directory.path());
    let expected = [
        finding("dates_no_guard"),
        finding("dates_wrong_guard"),
        finding("overflowing"),
        finding("percent"),
        finding("result"),
        finding("width"),
    ];

    assert_eq!(doctor_async_findings(&doctor, "src/Unsafe.vue"), expected);
    assert_eq!(lint_async_findings(&lint, "src/Unsafe.vue"), expected);
}

fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    write(directory.path(), "src/Accepted.vue", ACCEPTED);
    write(directory.path(), "src/Unsafe.vue", UNSAFE);
    directory
}

fn doctor(root: &Path) -> DoctorReport {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["doctor", "src", "--format", "json", "--exit-zero"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn lint(root: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args([
            "lint",
            "src/**/*.vue",
            "--no-config",
            "--cross-file",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn doctor_async_findings(report: &DoctorReport, path: &str) -> Vec<String> {
    let mut findings = report
        .findings()
        .iter()
        .filter(|finding| {
            finding.code == "VIZE_DOCTOR_CF_ASYNC_BOUNDARY" && finding.primary.path == path
        })
        .map(|finding| finding.message.to_string())
        .collect::<Vec<_>>();
    findings.sort_unstable();
    findings
}

fn lint_async_findings(report: &serde_json::Value, path: &str) -> Vec<String> {
    let prefix = "[vize:cross-file] vize:croquis/cf/async-boundary: ";
    let mut findings = report
        .as_array()
        .unwrap()
        .iter()
        .filter(|file| file["file"] == path)
        .flat_map(|file| file["messages"].as_array().unwrap())
        .filter(|message| message["ruleId"] == "cross-file")
        .filter_map(|message| message["message"].as_str()?.strip_prefix(prefix))
        .map(str::to_string)
        .collect::<Vec<_>>();
    findings.sort_unstable();
    findings
}

fn finding(variable: &str) -> String {
    format!(
        "Reactive state '{variable}' is mutated from an async boundary; stale completions can overwrite newer state"
    )
}

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
