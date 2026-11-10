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
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", label, (undefined as unknown as __VProp0$aria_label))").count(),
            1,
            "only the mapped diagnostic literal may reuse its checked contract: {text}"
        );
        let raw_call = "...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", label)";
        assert_eq!(
            text.matches(raw_call).count(),
            2,
            "the template slot scope must retain its two raw inference copies: {text}"
        );
        // Nothing in the slot is checked, so it has no props replay.
        assert_eq!(
            text.matches("// Component props in v-slot scope").count(),
            0,
            "{text}"
        );
        for (start, end, copies) in [(
            "// v-slot scope: #default",
            "// Component props type declarations",
            2,
        )] {
            assert_eq!(text.matches(start).count(), 1, "unique scope start: {text}");
            assert_eq!(text.matches(end).count(), 1, "unique scope end: {text}");
            let (_, after_start) = text.split_once(start).expect("the slot scope exists");
            let (scope, _) = after_start.split_once(end).expect("the slot scope ends");
            assert_eq!(
                scope.matches(raw_call).count(),
                copies,
                "the constructor and factory in {start} retain their raw value only where the scope declares its own host: {text}"
            );
        }
        assert_eq!(
            text.matches("...__vizeComponentAriaPropEntries(\"aria-label\", \"ariaLabel\", true)")
                .count(),
            1,
            "a bare attribute has no per-prop value owner and must remain checked: {text}"
        );
        assert_eq!(
            text.matches("true, (undefined as unknown as __VProp1$aria_label)")
                .count(),
            0,
            "an unowned attribute cannot borrow a phantom contract: {text}"
        );
    }
}
