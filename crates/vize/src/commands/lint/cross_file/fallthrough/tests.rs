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
<template>日本語<Dialog class="overlay" /></template>"#;
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
    let lowercase = diagnostics(
        parent,
        "<template><teleport to=\"body\"><div /></teleport></template>",
    );
    assert_eq!(lowercase.len(), 1, "{lowercase:?}");
    assert_eq!(
        lowercase[0].1 as usize,
        parent.find("class=\"overlay\"").unwrap()
    );
}

#[test]
fn structural_roots_and_script_use_attrs_are_classified_from_syntax() {
    let parent = "<script setup>import Dialog from './Dialog.vue'</script><template><Dialog class=\"x\" :style=\"style\" /></template>";
    for child in [
        "<script setup>// useAttrs()\n</script><template><Teleport to=\"body\"><div /></Teleport></template>",
        "<script setup>const note = 'useAttrs()'</script><template><Teleport to=\"body\"><div /></Teleport></template>",
    ] {
        assert_eq!(diagnostics(parent, child).len(), 2, "{child}");
    }
    assert!(diagnostics(parent, "<script setup>useAttrs()</script><template><Teleport to=\"body\"><div /></Teleport></template>").is_empty());
    for call in [
        "import { useAttrs as attrs } from 'vue'; attrs()",
        r"import { useAttrs as attrs } from 'vue'; \u0061ttrs()",
        "attrs(); import { useAttrs as attrs } from 'vue'",
        "import * as Vue from 'vue'; Vue.useAttrs()",
        "Vue.useAttrs(); import * as Vue from 'vue'",
    ] {
        let child = format!(
            "<script setup>{call}</script><template><Teleport to=\"body\"><div /></Teleport></template>"
        );
        assert!(diagnostics(parent, &child).is_empty(), "{child}");
    }
    let local = "<script setup>function useAttrs() {} useAttrs()</script><template><Teleport to=\"body\"><div /></Teleport></template>";
    assert_eq!(diagnostics(parent, local).len(), 2);
    let later_local = "<script setup>useAttrs(); function useAttrs() {}</script><template><Teleport to=\"body\"><div /></Teleport></template>";
    assert_eq!(diagnostics(parent, later_local).len(), 2);
    assert!(
        diagnostics(
            parent,
            "<template><div v-if=\"ready\" /><div v-else /></template>"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostics(
            parent,
            "<template><div v-for=\"item in items\" /></template>"
        )
        .len(),
        2
    );
}
