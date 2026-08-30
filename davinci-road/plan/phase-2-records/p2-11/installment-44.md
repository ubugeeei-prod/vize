# P2-11 Installment 44 - Single Nested Slot Wrapper Defaults

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5363](https://github.com/ubugeeei-prod/vize/pull/5363), merged
> 2026-08-30 at `1a717a959`.

This installment keeps unwrapped single nested slot templates on the default
slot path. The S2 slot pass now carries wrapper provenance into slot-region
consumption, and `createSlots` detection consults that provenance so a
`<template v-if>` or `<template v-for>` wrapper with one nested slot template
does not get mistaken for an explicit `createSlots` entry. Explicit `#slot
v-if` and `#slot v-for` carriers still use the shipped `createSlots` shape.

The durable witnesses are:

- [`emit_create_slots_wrappers.rs`](../../../../crates/vize_s1_to_s2/tests/emit_create_slots_wrappers.rs)
  - pins S1-to-S2 exact output for both single and multiple nested wrapper
    slot cases.
- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - keeps the S2 DOM output byte-identical to the shipped DOM lane for the
    wrapper-default cases.
- [`create_slots_walk.rs`](../../../../crates/vize_s1_to_s2/src/emit/create_slots_walk.rs)
  - centralizes the wrapper-aware `createSlots` predicate used by component
    emission.

This installment does not tick P2-11. It closes the single nested slot wrapper
default witness gap, but the production-lane switch, hydrated zero-divergence
corpus run and remaining patch-flag program stay open.
