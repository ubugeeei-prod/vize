use super::{check, resolve_test_tsgo_binary};

#[test]
fn unextracted_options_props_survive_comments_mixins_and_models() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, script) in [
        (
            "comment",
            "export /* component */ default { props: { initial: Number } }",
        ),
        ("newline", "export\ndefault { props: { initial: Number } }"),
        (
            "mixin",
            "const M = { props: { initial: Number } }; export default { mixins: [M] }",
        ),
    ] {
        for model in [false, true] {
            let setup = if model {
                "<script setup lang=\"ts\">const value = defineModel<string>();</script>"
            } else {
                ""
            };
            let child = vize_carton::cstr!(
                "<script lang=\"ts\">{script}</script>{setup}<template><i /></template>"
            );
            assert_eq!(
                check(
                    vize_carton::cstr!("unextracted-options-{name}-{model}").as_str(),
                    r#"{ "strictTemplates": true }"#,
                    &[
                        ("src/Child.vue", child.as_str()),
                        (
                            "src/App.vue",
                            r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :initial="1" /></template>"#
                        ),
                    ],
                ),
                [],
                "{name}, model={model}",
            );
        }
    }
}

#[test]
fn a_component_root_forwards_camelized_external_listener_keys() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    assert_eq!(
        check(
            "fallthrough-external-hyphenated-listener",
            r#"{ "strictTemplates": true, "fallthroughAttributes": true }"#,
            &[
                (
                    "src/External.ts",
                    r#"import { defineComponent } from "vue";
export default defineComponent({ emits: ["scroll-top"] });"#
                ),
                (
                    "src/Wrapper.vue",
                    r#"<script setup lang="ts">
import External from "./External";
defineEmits(["ready"]);
</script>
<template><External /></template>"#
                ),
                (
                    "src/App.vue",
                    r#"<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper @scroll-top="() => {}" /></template>"#
                ),
            ],
        ),
        [],
    );
}
