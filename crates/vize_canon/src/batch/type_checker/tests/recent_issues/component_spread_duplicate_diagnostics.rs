//! Named component props keep their per-attribute diagnostic when the usage
//! also contains a whole-object `v-bind` spread (#10069).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn named_prop_before_spread_reports_once_with_or_without_listener() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-spread-duplicate-diagnostics",
        &[
            (
                "src/MyInput.vue",
                r#"<script setup lang="ts">
defineProps<{ type?: string; label?: string }>();
defineEmits<{ blur: [event: FocusEvent] }>();
</script>

<template>
  <input :type :aria-label="label" @blur="$emit('blur', $event)" />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import MyInput from "./MyInput.vue";

const attrs = { label: "name" };
function onBlur(): void {}
</script>

<template>
  <MyInput :type="123" />
  <MyInput :type="123" v-bind="attrs" />
  <MyInput :type="123" @blur="onBlur" />
  <MyInput :type="123" v-bind="attrs" @blur="onBlur" />
  <MyInput v-bind="{ type: 1 }" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let actual: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| (file.as_str().replace('\\', "/"), *code, message.as_str()))
        .collect();
    let spread_origin: Vec<_> = actual
        .iter()
        .filter(|(_, _, location)| location.starts_with("13:"))
        .collect();
    assert_eq!(spread_origin.len(), 1, "{snapshot:#?}");
    let (file, code, message) = spread_origin[0];
    assert_eq!(file, "src/App.vue");
    assert_eq!(*code, Some(2322));
    assert!(
        message.starts_with("13:4:error ")
            && message.contains("number")
            && message.contains("string"),
        "the invalid type authored inside v-bind must retain its complete whole-props diagnostic: {snapshot:#?}"
    );
    let named_props: Vec<_> = actual
        .into_iter()
        .filter(|(_, _, location)| !location.starts_with("13:"))
        .map(|(file, code, message)| {
            (
                file,
                code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();
    assert_eq!(
        named_props,
        vec![
            ("src/App.vue".to_string(), Some(2322), "10:13".to_string()),
            ("src/App.vue".to_string(), Some(2322), "11:13".to_string()),
            ("src/App.vue".to_string(), Some(2322), "12:13".to_string()),
            ("src/App.vue".to_string(), Some(2322), "9:13".to_string()),
        ],
        "one authored prop error must produce one attribute-local diagnostic: {snapshot:#?}"
    );
}
