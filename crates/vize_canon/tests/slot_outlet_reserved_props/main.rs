#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

/// Nuxt UI's `Link.vue` forwards its `as` prop through `<slot v-bind="{ as }">`.
/// `as` is a reserved spelling, so the shorthand must read the prop binding
/// (vue-tsc: no TS18004) and still carry the prop type into the slot payload.
#[test]
fn slot_outlet_spread_shorthand_reads_a_reserved_prop_name() {
    let link = r#"<script setup lang="ts">
withDefaults(defineProps<{ as?: string; type?: string }>(), { as: 'button' })
</script>
<template><slot v-bind="{ as, type }" /></template>"#;
    let parent = r#"<script setup lang="ts">
import Link from './Link.vue'
</script>
<template><Link v-slot="{ as }">{{ as.toUpperCase() }}</Link></template>"#;
    assert_eq!(
        project::check(&[("src/Link.vue", link), ("src/Parent.vue", parent)]),
        [] as [String; 0]
    );
    let invalid = parent.replace("as.toUpperCase()", "as.toFixed()");
    assert_eq!(
        project::check(&[("src/Link.vue", link), ("src/Parent.vue", &invalid)]),
        [
            "src/Parent.vue(4,39): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"
        ]
    );
}
