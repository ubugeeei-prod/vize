# P2-11 Installment 30 - Inert Slot-Template Bindings

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5178](https://github.com/ubugeeei-prod/vize/pull/5178), merged
> 2026-08-28 at `2299a2114`.
> Issue: [#5177](https://github.com/ubugeeei-prod/vize/issues/5177).

This installment narrows the malformed slot-region boundary without changing
the shipped DOM path. Authored slot-template carriers may carry inert render
bindings from `v-once` and `v-memo`; the shipped lane does not surface those
directives in generated slot output, and the S2 DOM emitter now elides them
instead of treating the carrier as malformed.

The change stays local to slot-template realization:

- Plain, conditional and looped named slot templates accept inert `v-once` and
  `v-memo` bindings.
- Static attributes on authored slot templates remain elided from the emitted
  slot object, matching the shipped lane.
- The unsupported `SlotDefaultShape` bucket remains source-covered by `v-pin`
  on a slot template rather than by the now-inert render bindings.

The durable witnesses are:

- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - S2-vs-shipped byte fixtures for plain, conditional and looped slot-template
  carriers with inert render bindings.
- [`emit_slots.rs`](../../../../crates/vize_ricalco/tests/emit_slots.rs)
  - direct emitter coverage for named slot templates with inert bindings.
- [`emit_create_slots.rs`](../../../../crates/vize_ricalco/tests/emit_create_slots.rs)
  - direct createSlots coverage for conditional and looped slot templates.
- [`emit_unsupported_census.rs`](../../../../crates/vize_ricalco/tests/emit_unsupported_census.rs)
  and [`emit_unsupported_catalogue.rs`](../../../../crates/vize_ricalco/tests/emit_unsupported_catalogue.rs)
  - the refusal catalogue remains explicit and source-covered.

This installment does not tick P2-11. The production-lane switch, full-corpus
comparison count, remaining patch-flag program and DOM allocation budget remain
task-level gates.
