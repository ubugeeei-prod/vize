use super::{DoctorSource, analysis::analyze_application, discovery::discover_sources};
use std::{fs, path::Path};
use vize_carton::String;

#[test]
fn discovery_is_sorted_deduplicated_and_uses_standard_ignores() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/Z.vue",
        "<template><div /></template>",
    );
    write(directory.path(), "src/a.ts", "export const value = 1");
    write(directory.path(), "src/readme.md", "ignored");
    write(
        directory.path(),
        "node_modules/ignored.ts",
        "export const ignored = true",
    );

    let sources = discover_sources(directory.path(), &["src".into(), "src/a.ts".into()]).unwrap();
    let paths = sources
        .iter()
        .map(|source| source.path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(paths, ["src/Z.vue", "src/a.ts"]);
}

#[test]
fn duplicate_ids_use_authored_template_offsets() {
    let directory = tempfile::tempdir().unwrap();
    let first = r#"<script setup lang="ts">
const label = 'Email'
</script>
<template>
  <label for="email">{{ label }}</label>
  <input id="email" />
</template>
"#;
    let second = r#"<template>
  <input id="email" />
</template>
"#;
    let sources = vec![
        doctor_source("src/First.vue", first),
        doctor_source("src/Second.vue", second),
    ];

    let report = analyze_application(directory.path(), &sources, false).unwrap();
    let finding = report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_CF_DUPLICATE_ID")
        .unwrap();

    assert_eq!(finding.primary.path, "src/First.vue");
    assert_eq!(
        finding.primary.start as usize,
        first.find("id=\"email\"").unwrap()
    );
    assert!(finding.primary.end > finding.primary.start);
    assert_eq!(finding.related[0].location.path, "src/Second.vue");
    assert_eq!(
        finding.related[0].location.start as usize,
        second.find("id=\"email\"").unwrap()
    );
}

#[test]
fn split_script_diagnostics_map_back_to_script_setup() {
    let directory = tempfile::tempdir().unwrap();
    let parent = r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 0 })
</script>
<template><Child :item="state" /></template>
"#;
    let child = r#"<script lang="ts">
export const componentName = 'Child'
</script>
<script setup lang="ts">
const props = defineProps<{ item: { count: number } }>()
const { item } = props
</script>
"#;
    let sources = vec![
        doctor_source("src/Child.vue", child),
        doctor_source("src/Parent.vue", parent),
    ];

    let report = analyze_application(directory.path(), &sources, false).unwrap();
    let finding = report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_CF_DESTRUCTURING_BREAKS_REACTIVITY")
        .unwrap();
    let expected = child.rfind("props").unwrap();

    assert_eq!(finding.primary.path, "src/Child.vue");
    assert_eq!(finding.primary.start as usize, expected);
    assert_eq!(&child[expected..finding.primary.end as usize], "props");
}

#[test]
fn serialized_report_is_deterministic_for_input_order() {
    let directory = tempfile::tempdir().unwrap();
    let a = doctor_source("src/A.vue", "<template><div id=\"shared\" /></template>");
    let b = doctor_source("src/B.vue", "<template><div id=\"shared\" /></template>");
    let first = analyze_application(directory.path(), &[a, b], false).unwrap();
    let second = analyze_application(
        directory.path(),
        &[
            doctor_source("src/B.vue", "<template><div id=\"shared\" /></template>"),
            doctor_source("src/A.vue", "<template><div id=\"shared\" /></template>"),
        ],
        false,
    )
    .unwrap();

    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
}

fn doctor_source(path: &str, source: &str) -> DoctorSource {
    DoctorSource {
        path: path.into(),
        source: String::from(source),
    }
}

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
