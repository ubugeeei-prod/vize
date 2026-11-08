use super::super::temp_compile_project_dir;
use crate::types::{ScriptCompileOptions, SfcCompileOptions, TemplateCompileOptions};
use crate::{SfcParseOptions, compile_sfc, parse_sfc};
use std::fs;
use vize_carton::ToCompactString;

#[test]
fn imported_type_props_destructure_declares_runtime_props() {
    let project = temp_compile_project_dir("imported-type-props-destructure");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("timetable.ts"),
        r#"export interface TimetableCell {
  type: string
  title: string
  speakers?: string[]
}"#,
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import type { TimetableCell } from "./timetable"

const { type, title, speakers } = defineProps<TimetableCell>()
</script>

<template>
  <div :class="type">
    <h2>{{ title }}</h2>
    <span v-if="speakers">{{ speakers.length }}</span>
  </div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            id: Some(
                project
                    .join("Layout.vue")
                    .to_string_lossy()
                    .as_ref()
                    .to_compact_string(),
            ),
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result
            .code
            .contains("type: { type: String, required: true }"),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("title: { type: String, required: true }"),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("speakers: { type: Array, required: false }"),
        "{}",
        result.code
    );
    assert!(result.code.contains("__props.title"), "{}", result.code);
    assert!(result.code.contains("__props.speakers"), "{}", result.code);

    let _ = fs::remove_dir_all(project);
}
