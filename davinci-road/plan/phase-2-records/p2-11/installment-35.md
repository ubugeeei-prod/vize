# P2-11 Installment 35 - V-cloak DOM Cloak Markers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5205](https://github.com/ubugeeei-prod/vize/pull/5205), merged
> 2026-08-29 at `02c4eb1a7`.
> Issue: [#5204](https://github.com/ubugeeei-prod/vize/issues/5204).

This installment gives `v-cloak` a typed S2 representation and realizes it in
the S2 DOM emitter without flipping the shipped compiler lane. Vue compiler
removes `v-cloak` from generated props instead of resolving a runtime
directive, so the S2 op is a presence-only `vue.cloak` marker that preserves
the authored span for consumers while remaining inert for DOM output.

The lowering admits every compiler-compatible `Head::Cloak` spelling as the
same marker: bare `v-cloak`, value-bearing `v-cloak="x"`, argument and modifier
spellings, and dynamic-argument spellings. The lint rule that rejects those
spelled forms remains separate from compiler lowering.

The realized cases are:

- Native elements with empty, static-child, interpolation-child, `v-if`, and
  `v-for` bodies.
- Bare, value-bearing, argument, modifier and dynamic-argument `v-cloak`.
- Static-name components and `<slot>` outlets.
- `v-bind` object spread, `:id`, dynamic `:style`, custom directives, and
  native `v-model` combined with `v-cloak`.
- The shipped `v-for` empty-props marker for inert directives.

The durable witnesses are:

- [`folio_cloak.rs`](../../../../crates/vize_s2/tests/folio_cloak.rs)
  - exact Folio parse/print and owned-mirror coverage for `vue.cloak`.
- [`lowering_vcloak.rs`](../../../../crates/vize_ricalco/tests/lowering_vcloak.rs)
  - element and slot-outlet `v-cloak` lower to `vue.cloak`, including
    value-bearing, argument, modifier and dynamic-argument spellings.
- [`lowering_elements.rs`](../../../../crates/vize_ricalco/tests/lowering_elements.rs)
  - `v-pre` remains the unmapped-directive witness outside the dedicated
    directive-realization files.
- [`davinci_s2_cloak.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_cloak.rs)
  - S2-vs-shipped byte fixtures plus per-node patch-flag extraction across the
    realized cases above.
- [`op_family.rs`](../../../../crates/vize_s2/tests/op_family.rs)
  - the S2 attached-op canary covers the new `vue.cloak` binding variant
    without a wildcard.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
