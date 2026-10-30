use super::*;

#[test]
fn shared_aria_witness_emits_the_callable_three_argument_helper_without_alias_capture() {
    let source = r#"<script setup lang="ts">
import Child from './Child.vue';
type PropertyKey = 0;
type $NotUndefined<T> = never;
</script>
<template><Child aria-label="x" :size="false" /></template>
"#;
    for shared_virtual_module in [false, true] {
        let result = generate_vue_content_mapper_transform_with_options(
            Path::new("Parent.vue"),
            source,
            ContentMapperTransformOptions::default()
                .with_external_component_attrs_module(true)
                .with_external_component_type_helpers(true)
                .with_shared_virtual_module(shared_virtual_module),
        )
        .expect("installed shared-helper transform");
        let text = result.text.as_str();
        assert_eq!(
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", \"x\")")
                .count(),
            1,
            "the literal must call the emitted helper: {text}"
        );
        let narrowing = if shared_virtual_module {
            "T extends undefined ? never : T"
        } else {
            "$NotUndefined<T>"
        };
        let helper = format!(
            "function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>(raw: R, camel: C, value: T): {{ readonly [K in R | C]: {narrowing} }} {{ return {{ [raw]: value, [camel]: value }} as unknown as {{ readonly [K in R | C]: {narrowing} }}; }}"
        );
        assert_eq!(
            text.matches(helper.as_str()).count(),
            1,
            "the three-argument helper must preserve literal values without capturing the authored alias: {text}"
        );
        assert_eq!(
            text.matches("function __vizeComponentAriaPropValue")
                .count(),
            0
        );
        assert_eq!(
            text.matches("import type {} from \"vize/virtual-ts\";")
                .count(),
            usize::from(shared_virtual_module),
            "the requested embedded or installed declaration route must execute"
        );
        assert_eq!(
            text.matches("type PropertyKey = 0;").count(),
            1,
            "the authored shadow must remain present: {text}"
        );
        assert_eq!(
            text.matches("__vizeComponentAriaPropEntries<const R extends PropertyKey")
                .count(),
            0
        );
    }
}
