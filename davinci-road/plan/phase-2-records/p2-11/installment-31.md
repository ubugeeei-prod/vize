# P2-11 Installment 31 - Inline Slot-Template Carriers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5183](https://github.com/ubugeeei-prod/vize/pull/5183), merged
> 2026-08-28 at `f5aa60553`.
> Issue: [#5182](https://github.com/ubugeeei-prod/vize/issues/5182).

This installment closes the named malformed slot fact gap in the S2 DOM emitter
without flipping the shipped compiler lane. Slot-template carriers that are not
direct children of the owning component's slot group are not emitted as real DOM
`<template>` elements. They follow the shipped inline-template fallback instead.

The realized cases are:

- Nested slot-template carriers inside a component slot body.
- Conditional and looped `createSlots` entries whose slot body contains another
  slot-template carrier.
- Stray slot-template carriers in native children.
- Slot outlet fallback arrays containing a stray slot-template carrier.
- Empty, single-interpolation and multiple-child carrier bodies, preserving the
  shipped bare-expression and single-inline-array shapes.

The durable witnesses are:

- [`emit_slot_template_carriers.rs`](../../../../crates/vize_ricalco/tests/emit_slot_template_carriers.rs)
  - exact direct S2 emitter output for interpolation, multi-child, native-child
  and createSlots carrier cases.
- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - S2-vs-shipped byte fixtures for nested, stray, slot-outlet fallback and
  dynamic-name-hole cases.
- [`croquis-consumption.md`](../../croquis-consumption.md)
  - regenerated so the new `Croquis.bindings` reference stays accounted for.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
