#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

const BUTTON: &str = r#"<script setup lang="ts">
defineProps<{ onClick?: ((event: MouseEvent) => void) | Array<((event: MouseEvent) => void)> }>()
</script>
<template><button /></template>"#;
const EMITTER: &str = r#"<script setup lang="ts">
defineEmits<{ click: [event: MouseEvent] }>()
</script>
<template><button /></template>"#;

fn parent(child: &str, handler: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport Child from './{child}.vue'\nconst handlers = undefined as unknown as {handler}\n</script>\n<template><Child @click=\"handlers\" /></template>"
    )
}

/// Nuxt UI's Button declares `onClick?: Fn | Fn[]` and PricingPlan forwards
/// `props.button?.onClick`; vue-tsc checks the listener against that prop.
#[test]
fn a_listener_prop_that_admits_arrays_accepts_an_array_of_listeners() {
    let accepted = parent(
        "Button",
        "((event: MouseEvent) => void) | Array<(event: MouseEvent) => void> | undefined",
    );
    assert_eq!(
        project::check(&[("src/Button.vue", BUTTON), ("src/Parent.vue", &accepted)]),
        [] as [String; 0]
    );
    // The payload is still checked against the function member.
    let wrong_payload = parent("Button", "Array<(event: KeyboardEvent) => void>");
    let rows = project::check(&[
        ("src/Button.vue", BUTTON),
        ("src/Parent.vue", &wrong_payload),
    ]);
    assert_eq!(
        rows,
        [
            "src/Parent.vue(5,19): error TS2322: Type '((event: KeyboardEvent) => void)[]' is not assignable to type '((event: MouseEvent) => void)[] | ((event: MouseEvent) => any) | null | undefined'.\nType '((event: KeyboardEvent) => void)[]' is not assignable to type '((event: MouseEvent) => void)[]'.\nType '(event: KeyboardEvent) => void' is not assignable to type '(event: MouseEvent) => void'.\nTypes of parameters 'event' and 'event' are incompatible.\nType 'MouseEvent' is missing the following properties from type 'KeyboardEvent': charCode, code, isComposing, key, and 8 more.",
        ]
    );
    // A declared emit has no array form, so an array stays an error.
    let emitted = parent("Emitter", "Array<(event: MouseEvent) => void>");
    let rows = project::check(&[("src/Emitter.vue", EMITTER), ("src/Parent.vue", &emitted)]);
    assert_eq!(
        rows,
        [
            "src/Parent.vue(5,19): error TS2322: Type '((event: MouseEvent) => void)[]' is not assignable to type '(event: MouseEvent) => any'.\nType '((event: MouseEvent) => void)[]' provides no match for the signature '(event: MouseEvent): any'.",
        ]
    );
}
