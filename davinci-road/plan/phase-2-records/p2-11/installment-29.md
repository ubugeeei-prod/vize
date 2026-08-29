# P2-11 Installment 29 — Bare Template Default Slots

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5011](https://github.com/ubugeeei-prod/vize/pull/5011), merged
> 2026-08-26 at `3565326fe`.
> Issue: [#5010](https://github.com/ubugeeei-prod/vize/issues/5010).

This installment narrows the malformed slot-region remainder without changing
the shipped DOM path. A bare HTML `<template>` child of a component is not a
slot template: Vue keeps it as ordinary implicit default-slot content, and the
S2 DOM lane now does the same. Authored slot templates still require their
`ui.slot-content` binding, and malformed slot-template bindings remain a typed
`SlotDefaultShape` refusal.

The createSlots helper no longer re-discovers `ui.slot-content` after the
walker has already proven the child is a slot template. The walker threads the
borrowed `SlotContentOp` into entry emission, retiring the
`CreateSlotsMissingSlotTemplate` guard bucket while keeping the stable reason
key available for historical census output.

The durable witnesses are:

- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  — `bare_template_default` compares the S2 DOM lane against the shipped lane
  byte-for-byte.
- [`emit_slots.rs`](../../../../crates/vize_s1_to_s2/tests/emit_slots.rs)
  — the ricalco emitter pins the hoisted `_createElementVNode("template", ...)`
  default-slot output.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  — `SlotDefaultShape` remains source-covered by a real malformed slot
  template rather than the now-supported bare default child.
- [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  — `CreateSlotsMissingSlotTemplate` is accounted as retired, not guard-only.

This installment does not tick P2-11. Malformed slot fact gaps, the production
lane switch, full-corpus comparison count, patch-flag program, and DOM
allocation budget remain task-level gates.
