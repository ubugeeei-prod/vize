//! A `defineProps` result read only by the template is consumed, not unused.
//!
//! The template scope declares a synthetic `const props` so expressions can
//! spell `props.propName`; that shadow captured the in-closure `void props;`
//! anchor, so a setup `const props = defineProps(...)` whose only reads are in
//! the template — reka-ui forwards every component's props with
//! `v-bind="props"` — reported a false `TS6133`. The anchor now lands at setup
//! scope, before the shadow exists, under the same template-referenced
//! narrowing as the rest of the anchor list (vue-tsc reports nothing here).

use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

#[test]
fn define_props_result_read_only_by_template_v_bind_is_not_unused() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "define-props-result-template-v-bind-read",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
interface Props {
  label: string
}

const props = withDefaults(defineProps<Props>(), {
  label: 'fallback',
})
const orphan = 1
</script>

<template>
  <button v-bind="props"></button>
</template>
"#,
        )],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        !snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(6133) && message.contains("'props'")
        }),
        "template v-bind=\"props\" reads the defineProps result; TS6133 is a false positive, got: {snapshot:#?}"
    );
    // The same fixture keeps a genuinely unused binding flagged, so the anchor
    // cannot have suppressed unused-local reporting wholesale.
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(6133) && message.contains("'orphan'")
        }),
        "an unused setup binding must still report TS6133, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
