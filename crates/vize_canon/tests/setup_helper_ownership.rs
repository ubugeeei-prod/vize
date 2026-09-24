#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn authored_macro_named_values_keep_their_own_types() {
    let source = r#"<script setup lang="ts">
const defineProps = {};
const defineEmits = {};
const defineExpose = {};
const defineSlots = {};
const defineModel = {};
const defineOptions = {};
const withDefaults = {};
const useTemplateRef = (value: number) => value;
const result: number = useTemplateRef(1);
</script>
<template>{{ defineProps }}{{ defineEmits }}{{ defineExpose }}{{ defineSlots }}{{ defineModel }}{{ defineOptions }}{{ withDefaults }}{{ result }}</template>"#;
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [] as [String; 0]
    );
    let wrong = source.replace("const result: number", "const result: string");
    assert_eq!(
        project::check(&[("src/App.vue", &wrong)]),
        ["src/App.vue(10,7): error TS2322: Type 'number' is not assignable to type 'string'."]
    );
}

#[test]
fn ordinary_script_ambient_signatures_preserve_local_type_parameters() {
    let source = r#"<script lang="ts">
declare const identity: <T>(value: T, other?: typeof value) => T;
export const result: number = identity(1);
export { identity };
</script>"#;
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn type_only_macro_names_do_not_shadow_values() {
    let source = r#"<script setup lang="ts">
type defineProps = { title: string };
const props = defineProps<defineProps>();
const title: string = props.title;
</script><template>{{ title }}</template>"#;
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn leading_nocheck_survives_script_projection() {
    for source in [
        "<script setup lang=\"ts\">\n// @ts-nocheck\nmissing;\n</script>",
        "<script lang=\"ts\">\n// @ts-nocheck\nmissing;\n</script>",
        "<script lang=\"ts\">export const value = 1;</script><script setup lang=\"ts\">\n// @ts-nocheck\nmissing;\n</script>",
    ] {
        assert_eq!(
            project::check(&[("src/App.vue", source)]),
            [] as [String; 0],
            "{source}"
        );
    }
}
