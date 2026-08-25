use super::{DoctorSource, analysis::analyze_application, discovery::discover_sources};
use std::{fs, path::Path};
use vize_doctor::ContentFingerprint;
use vize_s0::String;

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
    assert_eq!(
        finding.provenance.invalidation_fingerprints["src/First.vue"],
        ContentFingerprint::digest(first)
    );
    assert_eq!(
        finding.provenance.invalidation_fingerprints["src/Second.vue"],
        ContentFingerprint::digest(second)
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
fn component_prop_contracts_map_template_and_script_setup_coordinates() {
    let directory = tempfile::tempdir().unwrap();
    let parent = r#"<script lang="ts">
export const parentName = 'Parent'
</script>
<script setup lang="ts">
import Counter from './Child.vue'
</script>
<template><Counter count="1" /></template>
"#;
    let child = r#"<script lang="ts">
export const componentName = 'Child'
</script>
<script setup lang="ts">
defineProps({ count: { type: Number, required: true } })
</script>
<template><span /></template>
"#;
    let report = analyze_application(
        directory.path(),
        &[
            doctor_source("src/Child.vue", child),
            doctor_source("src/Parent.vue", parent),
        ],
        false,
    )
    .unwrap();
    let finding = report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_CF_PROP_TYPE_MISMATCH")
        .unwrap();

    assert_eq!(finding.primary.path, "src/Parent.vue");
    assert_eq!(
        &parent[finding.primary.start as usize..finding.primary.end as usize],
        "count=\"1\""
    );
    assert_eq!(finding.related.len(), 1);
    assert_eq!(finding.related[0].location.path, "src/Child.vue");
    assert_eq!(
        finding.related[0].location.start as usize,
        child.find("defineProps").unwrap()
    );
}

#[test]
fn component_prop_contracts_map_missing_and_undeclared_template_spans() {
    let directory = tempfile::tempdir().unwrap();
    let parent = r#"<script setup lang="ts">
import ChildCard from './Child.vue'
</script>
<template><ChildCard extra="value" /></template>
"#;
    let child = r#"<script setup lang="ts">
defineProps<{ id: number }>()
</script>
<template><span /></template>
"#;
    let report = analyze_application(
        directory.path(),
        &[
            doctor_source("src/Parent.vue", parent),
            doctor_source("src/Child.vue", child),
        ],
        false,
    )
    .unwrap();
    let missing = report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_CF_MISSING_REQUIRED_PROP")
        .unwrap();
    let undeclared = report
        .findings()
        .iter()
        .find(|finding| finding.code == "VIZE_DOCTOR_CF_UNDECLARED_PROP")
        .unwrap();

    assert_eq!(
        &parent[missing.primary.start as usize..missing.primary.end as usize],
        "<ChildCard extra=\"value\" />"
    );
    assert_eq!(
        missing.related[0].location.start as usize,
        child.find("defineProps").unwrap()
    );
    assert_eq!(
        &parent[undeclared.primary.start as usize..undeclared.primary.end as usize],
        "extra=\"value\""
    );
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
