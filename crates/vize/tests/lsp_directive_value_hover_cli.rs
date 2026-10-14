#![cfg(test)]
//! `v-html` and `v-text` values are template expressions the checker types,
//! like every other directive value (#9110).
use serde_json::json;

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::Fixture;

const APP: &str = "<script setup lang=\"ts\">
import { computed, ref } from \"vue\";
const html = ref(\"<b>a</b>\");
const text = computed(() => \"plain\");
const shown = ref(true);
</script>

<template>
  <div v-html=\"html\" />
  <p v-text=\"text\" />
  <span v-show=\"shown\">{{ text }}</span>
</template>
";

#[test]
fn v_html_and_v_text_values_hover_with_their_checker_types() {
    let mut fixture = Fixture::new_with_vue(APP);
    fixture.open(APP);
    assert_eq!(
        fixture.request("textDocument/hover", APP, "html\" />")["contents"],
        json!({"kind": "markdown", "value": "```typescript\nconst html: string\n```"})
    );
    assert_eq!(
        fixture.request("textDocument/hover", APP, "text\" />")["contents"],
        json!({"kind": "markdown", "value": "```typescript\nconst text: string\n```"})
    );
    assert_eq!(
        fixture.request("textDocument/hover", APP, "shown\">")["contents"],
        json!({"kind": "markdown", "value": "```typescript\nconst shown: boolean\n```"})
    );
    fixture.shutdown();
}
