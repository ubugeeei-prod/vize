#![cfg(test)]
//! `v-html` and `v-text` values are template expressions the checker types,
//! like every other directive value (#9110).
use crate::support::lsp_vue_project as support;
use serde_json::json;
use support::Fixture;

const APP: &str = "<script setup lang=\"ts\">
import { computed, ref } from \"vue\";
const html = ref(\"<b>a</b>\");
const text = computed(() => \"plain\");
const shown = ref(true);
function copy() {}
const paste = () => {};
</script>

<template>
  <div v-html=\"html\" />
  <p v-text=\"text\" />
  <span v-show=\"shown\">{{ text }}</span>
  <button @click=\"copy\" @paste=\"paste\" />
</template>
";

#[test]
fn v_html_and_v_text_values_hover_with_their_checker_types() {
    let mut fixture = Fixture::new_with_vue(APP);
    assert_eq!(fixture.open(APP), json!([]), "valid directive values");
    for (needle, symbol, expected) in [
        ("html\" />", "html", "const html: string"),
        ("text\" />", "text", "const text: string"),
        ("shown\">", "shown", "const shown: boolean"),
        ("copy\" @paste", "copy", "function copy(): void"),
        ("paste\" />", "paste", "const paste: () => void"),
    ] {
        // Anchor directly on the authored value. The original event-handler
        // probe split needles on '@', accidentally requesting a garbage offset.
        let start = support::position(APP, needle);
        let mut end = start.clone();
        end["character"] = json!(start["character"].as_u64().unwrap() + symbol.len() as u64);
        assert_eq!(
            fixture.request("textDocument/hover", APP, needle),
            json!({
                "contents": {"kind": "markdown", "value": vize_l0::cstr!("```typescript\n{expected}\n```")},
                "range": {"start": start, "end": end}
            }),
            "complete checker hover at {needle}"
        );
    }

    // Missing expression collection silently hid type errors as well as hover
    // types. Check both content directives, their authored ranges, and recovery.
    let invalid = APP
        .replace("v-html=\"html\"", "v-html=\"html.missingHtml\"")
        .replace("v-text=\"text\"", "v-text=\"text.missingText\"");
    let diagnostics = fixture.change(&invalid, 2);
    let diagnostics = diagnostics.as_array().unwrap();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    for property in ["missingHtml", "missingText"] {
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic["message"].as_str().unwrap().contains(property))
            .unwrap_or_else(|| panic!("missing diagnostic for {property}: {diagnostics:#?}"));
        assert_eq!(diagnostic["code"], 2339);
        let start = support::position(&invalid, property);
        let mut end = start.clone();
        end["character"] = json!(start["character"].as_u64().unwrap() + property.len() as u64);
        assert_eq!(diagnostic["range"], json!({"start": start, "end": end}));
    }
    assert_eq!(fixture.change(APP, 3), json!([]), "fixed content values");
    fixture.shutdown();
}
