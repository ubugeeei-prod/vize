# P2-11 Installment 32 - V-show Runtime Directives

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5198](https://github.com/ubugeeei-prod/vize/pull/5198), merged
> 2026-08-28 at `2be66b0f0`.
> Issue: [#5197](https://github.com/ubugeeei-prod/vize/issues/5197).

This installment gives well-formed `v-show` a typed S2 representation and
realizes it in the S2 DOM emitter without flipping the shipped compiler lane.
The lowering admits only `v-show="expr"` with no argument or modifier; invalid
spellings still produce `defer.v-show` info diagnostics and keep the owner
fragment. The DOM emitter resolves `_vShow` and emits runtime directive entries
through `withDirectives`.

The realized cases are:

- Native elements with static text, interpolation, `v-if`, `v-for` and dynamic
  props.
- Root and child components, including the shipped `NEED_PATCH` component patch
  flag.
- Source-order parity with custom directives.
- Native `v-model` plus `v-show`, preserving update-listener prop flags.

The durable witnesses are:

- [`folio_show.rs`](../../../../crates/vize_s2/tests/folio_show.rs)
  - exact Folio parse/print and owned-mirror coverage for `vue.show`.
- [`lowering_elements.rs`](../../../../crates/vize_s1_to_s2/tests/lowering_elements.rs)
  - well-formed `v-show` lowers to `vue.show`; the deferral witness remains on
    a still-unmapped directive.
- [`davinci_s2_show.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_show.rs)
  - S2-vs-shipped byte fixtures plus per-node patch-flag extraction across the
    realized cases above.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  and [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  - non-JS `v-show` expressions remain an explicit, source-covered refusal.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
