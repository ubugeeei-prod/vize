# P2-11 Installment 14 — implicit slots and broad DOM families (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4742](https://github.com/ubugeeei-prod/vize/pull/4742) is the large
catch-up squash for the DOM S2 emit lane. Its title names implicit
default text slots, but the merged scope also records the broad DOM
surface required to compare those slots byte-for-byte with the shipped
lane.

The emitted families now named by `crates/vize_s1_to_s2/src/emit.rs` are:
implicit default slots (`withCtx`, `_: 1|2`, text and static-vnode
hoists), named / scoped `<template>` slots, `createSlots` for `v-if` /
`v-for` slot templates, slot outlets (`renderSlot`, `_: 3 FORWARDED`),
Vue builtins, `<component :is>`, template fragments, `<template v-if>` /
`<template v-for>` fragments, `v-model`, custom directives,
colon / vnode-hook events with duplicate-handler merging, destructured
`v-for` aliases, `createSlots` + `v-slots`, and dynamic `v-if` keys.

The PR also carried the scoped-SFC hoist correction needed for parity:
scoped component props stay inline rather than hoisted so scope attrs are
not baked into render props. The SFC regression lives with the compiler
tests and the DOM S2 witnesses live under `crates/vize_atelier_dom/tests`.

After this installment the module-level named unsupported families are
`.native` and filters. Several `EmitError::Unsupported` arms still guard
local malformed or not-yet-witnessed shapes; the series record keeps
those as the next audit list instead of pretending P2-11 is complete.
