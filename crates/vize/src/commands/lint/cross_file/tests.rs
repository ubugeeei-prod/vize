use super::{build_cross_file_lint_output_with_report, combine_cross_file_report};
use std::fs;
use vize_patina::HelpLevel;

#[test]
fn cross_file_complexity_report_mentions_hotspot_reason() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("App.vue");
    let child = dir.path().join("Child.vue");

    fs::write(&app, r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'
const ready = true
const enabled = true
const fallback = false
const state = reactive({ count: 0 })
</script>
<template><Child v-if="ready && enabled" :item="state" /><Child v-if="fallback" :item="state" /></template>
"#).unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
defineProps<{ item: { count: number } }>()
</script>
"#,
    )
    .unwrap();

    let files = [&app, &child]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output =
        build_cross_file_lint_output_with_report(&files, HelpLevel::Short, false, true, None);
    let report = output
        .complexity_report
        .as_deref()
        .expect("complexity report should be rendered");

    assert!(report.contains("## Cross-file Complexity"));
    assert!(report.contains("App.vue"));
    assert!(report.contains("template-control-flow"));
    assert!(report.contains("template cyclomatic=4"));
    assert!(report.contains("### Template Complexity"));
    assert!(report.contains("prop edges=2"));
}

#[test]
fn combined_cross_file_report_keeps_tree_before_complexity() {
    let report = combine_cross_file_report(Some("tree"), Some("complexity")).unwrap();

    assert_eq!(report, "tree\ncomplexity");
}
