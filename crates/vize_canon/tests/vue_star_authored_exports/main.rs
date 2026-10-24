#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

const ACCORDION: &str = r#"<script lang="ts">
export interface AccordionProps { multiple?: boolean }
export const accordionDefaults = { multiple: false };
</script>
<script setup lang="ts">
defineProps<AccordionProps>()
defineSlots<{ default(props: { open: boolean }): unknown }>()
</script>
<template><slot :open="true" /></template>"#;
const BUTTON: &str = r#"<script lang="ts">
export interface ButtonProps { label?: string }
</script>
<script setup lang="ts">
defineProps<ButtonProps>()
defineEmits<{ click: [] }>()
</script>
<template><button /></template>"#;

/// A Nuxt UI style barrel re-exports many SFCs by star. Like vue-tsc, only
/// their authored exports are public, so the generated `Props`/`Slots`/`Emits`
/// of each component must not collide (TS2308).
#[test]
fn star_barrels_of_components_with_authored_exports_only_expose_authored_names() {
    let barrel = "export * from './Accordion.vue';\nexport * from './Button.vue';\n";
    let consumer = "import { accordionDefaults, type AccordionProps, type ButtonProps } from './index';\nexport const props: AccordionProps = accordionDefaults;\nexport const button: ButtonProps = { label: 'x' };\n";
    assert_eq!(
        project::check(&[
            ("src/Accordion.vue", ACCORDION),
            ("src/Button.vue", BUTTON),
            ("src/index.ts", barrel),
            ("src/consumer.ts", consumer),
        ]),
        [] as [String; 0]
    );
    let invalid = consumer.replace("label: 'x'", "label: 1");
    assert_eq!(
        project::check(&[
            ("src/Accordion.vue", ACCORDION),
            ("src/Button.vue", BUTTON),
            ("src/index.ts", barrel),
            ("src/consumer.ts", &invalid),
        ]),
        ["src/consumer.ts(3,38): error TS2322: Type 'number' is not assignable to type 'string'."]
    );
}

/// Authored names that really collide keep both stars, so the authored TS2308
/// is still reported, and a barrel's own export keeps shadowing a star.
#[test]
fn authored_collisions_and_local_shadowing_keep_typescript_star_semantics() {
    let twin = BUTTON.replace("ButtonProps", "AccordionProps");
    let barrel = "export * from './Accordion.vue';\nexport * from './Twin.vue';\n";
    let diagnostics = project::check(&[
        ("src/Accordion.vue", ACCORDION),
        ("src/Twin.vue", &twin),
        ("src/index.ts", barrel),
    ]);
    // Both stars stay whole, so the generated aliases join the authored
    // collision exactly as they did before stars were narrowed.
    assert_eq!(
        diagnostics,
        [
            "src/index.ts(2,1): error TS2308: Module './Accordion.vue' has already exported a member named 'AccordionProps'. Consider explicitly re-exporting to resolve the ambiguity.",
            "src/index.ts(2,1): error TS2308: Module './Accordion.vue' has already exported a member named 'Emits'. Consider explicitly re-exporting to resolve the ambiguity.",
            "src/index.ts(2,1): error TS2308: Module './Accordion.vue' has already exported a member named 'Props'. Consider explicitly re-exporting to resolve the ambiguity.",
            "src/index.ts(2,1): error TS2308: Module './Accordion.vue' has already exported a member named 'Slots'. Consider explicitly re-exporting to resolve the ambiguity.",
        ]
    );
    let shadowed = "export * from './Accordion.vue';\nexport const accordionDefaults = 1;\n";
    assert_eq!(
        project::check(&[("src/Accordion.vue", ACCORDION), ("src/index.ts", shadowed)]),
        [] as [String; 0]
    );
}
