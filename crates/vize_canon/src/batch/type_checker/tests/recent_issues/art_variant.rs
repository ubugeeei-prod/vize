use vize_carton::String;

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn art_variant_markup_reports_the_same_prop_error_as_a_template() {
    // #7219: `<variant>` markup in an art file is type-checked with script-setup
    // bindings in scope. A wrong prop type errors the same way as a template.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "art-variant-prop-check",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><span>{{ label }}</span></template>
"#,
            ),
            (
                "src/Normal.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const title = "heading"
</script>
<template>
  <Child :label="1" />
  <span>{{ title }}</span>
</template>
"#,
            ),
            (
                "src/Button.art.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const title = "heading"
</script>
<art>
  <variant name="Broken">
    <Child :label="1" />
    <span>{{ title }}</span>
  </variant>
</art>
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let normal = prop_type_mismatch(&snapshot, "src/Normal.vue");
    let art = prop_type_mismatch(&snapshot, "src/Button.art.vue");
    // The art file wraps the same element in `<variant>`, so the authored
    // line differs. The contract is the same TS2322 text (#7219).
    assert_eq!(
        art, normal,
        "variant prop errors must match the template, got: {snapshot:#?}"
    );
    assert!(
        normal.is_some(),
        "expected a label prop mismatch, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Button.art.vue"
                && matches!(code, Some(2304 | 2339))
                && (message.contains("title") || message.contains("Child")))
        }),
        "script-setup bindings stay in scope inside variant markup, got: {snapshot:#?}"
    );
}

fn prop_type_mismatch(snapshot: &[(String, Option<u32>, String)], file: &str) -> Option<String> {
    snapshot.iter().find_map(|(candidate, code, message)| {
        if candidate == file && *code == Some(2322) && message.contains("not assignable") {
            message
                .split_once(":error ")
                .map(|(_, body)| String::from(body))
        } else {
            None
        }
    })
}
