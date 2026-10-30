use super::*;

#[test]
fn aria_contracts_only_accompany_owned_values_in_the_diagnostic_literal() {
    let source = r#"<script setup lang="ts">
import Child from './Child.vue';
const label = 'ok';
</script>
<template>
  <Child :aria-label="label" v-slot="{ value }">{{ value }}</Child>
  <Child aria-label />
</template>
"#;
    for shared in [false, true] {
        let transform = generate_vue_content_mapper_transform_with_options(
            Path::new("Parent.vue"),
            source,
            ContentMapperTransformOptions::default().with_shared_virtual_module(shared),
        )
        .expect("production ARIA and slot transform");
        let text = transform.text.as_str();
        let raw_return = if shared {
            "T extends undefined ? never : T"
        } else {
            "$NotUndefined<T>"
        };
        let raw_signature = format!(
            "function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>(raw: R, camel: C, value: T): {{ readonly [K in R | C]: {raw_return} }};"
        );
        assert_eq!(
            text.matches(raw_signature.as_str()).count(),
            1,
            "the raw overload must expose no expected-contract type to contextual inference: {text}"
        );
        assert_eq!(
            text.matches("function __vizeComponentAriaPropEntries")
                .count(),
            3,
            "raw and owned overloads precede their hidden implementation: {text}"
        );
        assert_eq!(
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", label, (undefined as unknown as __Child_0_prop_aria_label))").count(),
            1,
            "only the mapped diagnostic literal may reuse its checked contract: {text}"
        );
        assert_eq!(
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", label)")
                .count(),
            2,
            "both slot inference copies must keep the exact raw value: {text}"
        );
        assert_eq!(
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", true)")
                .count(),
            1,
            "a bare attribute has no per-prop value owner and must remain checked: {text}"
        );
        assert_eq!(
            text.matches("true, (undefined as unknown as __Child_1_prop_aria_label)")
                .count(),
            0,
            "an unowned attribute cannot borrow a phantom contract: {text}"
        );
    }
}
