#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use crate::support::lsp_vue_project as support;
use serde_json::{Value, json};
use support::Fixture;

const SOURCE: &str = r#"<script setup lang="ts">
import { computed, ref, watch } from "vue";

const count = ref(0);
const double = computed(() => count.value * 2);
watch(count, () => {});
    
</script>

<template>
  <span>{{ double }}</span>
</template>
"#;

#[test]
fn native_script_setup_completion_merges_checker_builtins_and_globals_by_label() {
    let mut fixture = Fixture::new_with_vue(SOURCE);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let completion = fixture.request("textDocument/completion", SOURCE, "    \n</script>");
    let items = completion
        .as_array()
        .or_else(|| completion["items"].as_array())
        .unwrap_or_else(|| panic!("missing completion list: {completion:#}"));

    for label in [
        "ref",
        "computed",
        "watch",
        "defineProps",
        "defineEmits",
        "defineOptions",
        "Comment",
        "Text",
        "compile",
        "compileToFunction",
        "stop",
    ] {
        assert_eq!(
            items.iter().filter(|item| item["label"] == label).count(),
            1,
            "completion count for {label}: {items:#?}",
        );
    }

    let r#ref = item(items, "ref");
    assert_eq!(r#ref["kind"], 6);
    assert_eq!(r#ref["sortText"], "11");
    assert_eq!(r#ref["detail"], "function ref<T>(value: T): Ref<T>");
    let define_props = item(items, "defineProps");
    assert_eq!(define_props["kind"], 3);
    assert_eq!(define_props["detail"], "Macro: defineProps<T>()");

    fixture.shutdown();
}

fn item<'a>(items: &'a [Value], label: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["label"] == label)
        .unwrap_or_else(|| panic!("missing {label}: {items:#?}"))
}
