#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

/// The shape vue-tsc emits for a generic SFC in a library (reka-ui's
/// `PinInputRoot`): a function whose return carries `__ctx`.
const LIBRARY: &str = r#"type PinValue<Type extends 'text' | 'number'> = [Type] extends ['number'] ? number[] : string[];
export declare const PinRoot: <Type extends 'text' | 'number' = 'text'>(
  props: import('vue').PublicProps & { type?: Type; modelValue?: PinValue<Type> | null; mask?: boolean } & (typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P } ? P : {}),
  ctx?: any,
) => import('vue').VNode & { __ctx?: { props: { type?: Type } } };
"#;

/// Nuxt UI's PinInput forwards its own generic value to the library root.
/// vue-tsc infers the root's `Type` from the call; extracting its props
/// instead fixed `Type` to the constraint and reported TS2322.
#[test]
fn generic_library_children_infer_their_parameters_from_the_bound_props() {
    let forwarding = r#"<script setup lang="ts" generic="T extends 'text' | 'number'">
import { PinRoot } from './library'
type PinValue<Type extends 'text' | 'number'> = [Type] extends ['number'] ? number[] : string[]
const props = defineProps<{ modelValue?: PinValue<T>; type?: T }>()
</script>
<template><PinRoot :type="props.type" :model-value="props.modelValue" data-slot="root" /></template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", LIBRARY), ("src/Pin.vue", forwarding)]),
        [] as [String; 0]
    );
    let wrong = r#"<script setup lang="ts">
import { PinRoot } from './library'
</script>
<template>
  <PinRoot type="number" :model-value="['a']" />
  <PinRoot :mask="'yes'" />
</template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", LIBRARY), ("src/Wrong.vue", wrong)]),
        [
            "src/Wrong.vue(5,41): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Wrong.vue(6,13): error TS2322: Type 'string' is not assignable to type 'boolean | undefined'.",
        ]
    );
}

/// An inline callback prop is contextually typed by the instantiated library
/// props (Nuxt UI's CommandPalette `:text-content="item => ..."`).
#[test]
fn inline_callbacks_of_generic_library_children_use_the_inferred_parameters() {
    let library = r#"export declare const List: <T>(
  props: import('vue').PublicProps & { options: T[]; textContent?: (option: T) => string } & (typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P } ? P : {}),
  ctx?: any,
) => import('vue').VNode & {
  __ctx?: { props: { options: T[]; textContent?: (option: T) => string } };
};
"#;
    let component = r#"<script setup lang="ts">
import { List } from './library'
const items = [{ label: 'a' }]
const loose = items as any[]
</script>
<template>
  <List :options="loose" :text-content="item => String(item)" />
  <List :options="items" :text-content="item => item.label.toFixed()" />
</template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", library), ("src/List.vue", component)]),
        [
            "src/List.vue(8,60): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
        ]
    );
}
