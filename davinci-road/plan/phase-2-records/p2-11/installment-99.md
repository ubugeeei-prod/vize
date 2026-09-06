# P2-11 Installment 99 - Scoped-CSS Scope Id

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5677](https://github.com/ubugeeei-prod/vize/pull/5677), merged
> 2026-09-04 as `df7a14809`.

This installment lands `scope_id`, which completes the DOM-relevant half of
`CodegenOptions` in `DomEmitOptions`. What remains outside the emitter's
surface is deliberate: `source_map` / `filename` belong to the structured S4
emitter (P3-9), and `ssr` belongs to the SSR lane (phase 3). A later selector
audit confirmed `optimize_imports` is a stale DOM no-op and should not block S2.

## Why this one is wide

`cache_handlers` had a single choke point. `scope_id` has none: `<style
scoped>` puts a `"data-v-abc123": ""` pair on every props object, and the
shipped lane builds props objects in several places. The S2 emitter mirrors
that shape, so all of those surfaces need the id:

- `emit::props_object` appends the pair last and counts it in the multiline
  decision.
- `emit::props_static::emit_inline` covers static-attributes-only paths.
- `emit::props_static::{root_hoist_props, component_hoist_props}` covers
  hoisted native and component prop objects.
- `emit::hoist` threads the id through the static-VNode string builders.
- `emit::merge` emits the pair once as a trailing `mergeProps` argument.

Three behaviours were only discoverable by running the comparator:

- A scope id un-refuses a prop-less hoist.
- A `v-if` branch key stops taking the key-only shortcut.
- A lone spread becomes a `mergeProps` call.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_scope_id.rs` compares the S2 emit
against the shipped lane byte-for-byte over 38 templates: empty and static
elements, static and dynamic binds, class/style, multi-root, deep and
sibling static trees, `v-if` / `v-else` / `v-for` and a `v-for` with a static
child, components with static, dynamic and no props, a component with a slot,
a slot outlet, a `<template>` root, foreign SVG, `v-once`, `v-html`,
`v-show`, `v-model`, `ref`, all three spread shapes, a dynamic bind key, and
`Teleport` / `KeepAlive`.

`the_option_is_what_produces_the_scope_attribute` pins the full render body
with the option off and on over the spread shape, so the trailing-argument
rule is asserted rather than assumed.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
