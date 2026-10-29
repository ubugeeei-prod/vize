//! Complete #10914 project proving composable results through the public CLI.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "whole CLI fixture")]

use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

#[test]
fn composable_results_are_not_reported_as_non_reactive_provides() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(
        root,
        "src/composables/selection.ts",
        r#"import { ref } from "vue";
import type { InjectionKey, Ref } from "vue";
export const SelectedKey: InjectionKey<Ref<string[]>> = Symbol("selected");
export const ToastKey: InjectionKey<{ message: Ref<string>; show(text: string): void }> = Symbol("toast");
export const UnknownKey: InjectionKey<unknown> = Symbol("unknown");
export const SnapshotKey: InjectionKey<string> = Symbol("snapshot");
export const PlainKey: InjectionKey<string> = Symbol("plain");
export function useSelection() { const selected = ref<string[]>([]); return { selected }; }
export function useToast() { const message = ref(""); return { message, show: (text: string) => (message.value = text) }; }
export function getUnknown(): unknown { return {}; }
"#,
    );
    write(
        root,
        "src/App.vue",
        r#"<script setup lang="ts">
import { provide, ref } from "vue";
import { SelectedKey, ToastKey, UnknownKey, SnapshotKey, PlainKey, useSelection, useToast, getUnknown } from "./composables/selection";
import Picker from "./components/Picker.vue";
const { selected } = useSelection();
const toast = useToast();
const unknown = getUnknown();
const source = ref("live");
const snapshot = source.value;
const plain = "fixed";
provide(SelectedKey, selected);
provide(ToastKey, toast);
provide(UnknownKey, unknown);
provide(SnapshotKey, snapshot);
provide(PlainKey, plain);
</script>
<template><Picker /></template>
"#,
    );
    write(
        root,
        "src/components/Picker.vue",
        r#"<script setup lang="ts">
import { inject } from "vue";
import { SelectedKey, ToastKey, UnknownKey, SnapshotKey, PlainKey } from "../composables/selection";
const selected = inject(SelectedKey)!;
const toast = inject(ToastKey)!;
const unknown = inject(UnknownKey);
const snapshot = inject(SnapshotKey);
const plain = inject(PlainKey);
</script>
<template><button @click="toast.show(selected.join(', '))">{{ toast.message.value }} {{ unknown }} {{ snapshot }} {{ plain }}</button></template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args([
            "lint",
            "--format",
            "json",
            "--no-config",
            "--cross-file",
            "src/**/*.vue",
        ])
        .output()
        .expect("run source-built vize lint");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let messages = report
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["messages"].as_array().unwrap())
        .filter_map(|message| message["message"].as_str())
        .filter(|message| message.contains("non-reactive-provide"))
        .collect::<Vec<_>>();

    assert_eq!(messages.len(), 2, "{messages:#?}");
    assert!(
        messages
            .iter()
            .any(|message| message.contains("SnapshotKey"))
    );
    assert!(messages.iter().any(|message| message.contains("PlainKey")));
    for live_key in ["SelectedKey", "ToastKey", "UnknownKey"] {
        assert!(
            messages.iter().all(|message| !message.contains(live_key)),
            "{live_key}: {messages:#?}"
        );
    }
}
