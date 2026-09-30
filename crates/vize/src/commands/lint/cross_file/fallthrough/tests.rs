use super::{super::build_cross_file_lint_output, RULE};
use std::{fs, path::Path};
use vize_patina::HelpLevel;

fn diagnostics(parent: &str, child: &str) -> Vec<(String, u32, u32)> {
    let dir = tempfile::tempdir().unwrap();
    let parent_path = dir.path().join("Parent.vue");
    let child_path = dir.path().join("Dialog.vue");
    fs::write(&parent_path, parent).unwrap();
    fs::write(&child_path, child).unwrap();
    let files = [&parent_path, &child_path]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, HelpLevel::Short, false);
    output
        .results
        .into_iter()
        .filter(|result| Path::new(result.filename.as_str()).file_name().unwrap() == "Parent.vue")
        .flat_map(|result| result.diagnostics)
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            (
                diagnostic.message.to_string(),
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

#[test]
fn warns_at_each_call_site_attr_for_teleport_without_flagging_props_or_emits() {
    let parent = r#"<script setup lang="ts">import Dialog from './Dialog.vue'</script>
<template><Dialog title="Hi" class="overlay" :style="styling" @close="hide" /></template>"#;
    let child = r#"<script setup lang="ts">
defineProps<{ title: string }>()
defineEmits(['close'])
</script>
<template><Teleport to="body"><div /></Teleport></template>"#;
    let found = diagnostics(parent, child);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        found[0].1 as usize,
        parent.find("class=\"overlay\"").unwrap()
    );
    assert_eq!(
        found[1].1 as usize,
        parent.find(":style=\"styling\"").unwrap()
    );
    assert!(
        found
            .iter()
            .all(|(message, _, _)| message.contains("Teleport root"))
    );
}

#[test]
fn covers_fragment_text_and_disabled_inheritance_but_allows_explicit_attrs() {
    let parent = r#"<script setup>import Dialog from './Dialog.vue'</script>
<template><Dialog class="overlay" /></template>"#;
    for child in [
        "<template><div /><div /></template>",
        "<template>{{ message }}</template>",
        "<script setup>defineOptions({ inheritAttrs: false })</script><template><div /></template>",
        "<script>export default { inheritAttrs: false }</script><template><div /></template>",
    ] {
        let found = diagnostics(parent, child);
        assert_eq!(found.len(), 1, "{child}: {found:?}");
        assert_eq!(
            found[0].1 as usize,
            parent.find("class=\"overlay\"").unwrap()
        );
    }
    for child in [
        "<template><div /></template>",
        "<template><Teleport to=\"body\"><div v-bind=\"$attrs\" /></Teleport></template>",
    ] {
        assert!(diagnostics(parent, child).is_empty(), "{child}");
    }
}
