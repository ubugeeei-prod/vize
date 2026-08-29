# P2-11 Installment 34 - V-text Text-Content Props

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5203](https://github.com/ubugeeei-prod/vize/pull/5203), merged
> 2026-08-29 at `11750115a`.
> Issue: [#5202](https://github.com/ubugeeei-prod/vize/issues/5202).

This installment gives `v-text` a typed S2 representation and realizes it in
the S2 DOM emitter without flipping the shipped compiler lane. The lowering
admits element and slot-outlet `v-text` as `vue.text`, including value-less
spellings so DOM emit can preserve the shipped
`textContent: _toDisplayString(undefined)` shape. Argument and modifier
spellings still produce `defer.v-text` info diagnostics and keep the owner
fragment.

The realized cases are:

- Native elements with empty, static-child, interpolation-child, `v-if`, and
  `v-for` bodies.
- Value-bearing, value-less, and empty-string `v-text`.
- Static-name components and `<slot>` outlets.
- `v-bind` object spread, `:id`, dynamic `:style`, custom directives, and
  native `v-model` combined with `v-text`.
- Non-JS expressions as a source-covered `text_directive_expression_not_js`
  refusal.

The durable witnesses are:

- [`folio_text.rs`](../../../../crates/vize_s2/tests/folio_text.rs)
  - exact Folio parse/print and owned-mirror coverage for value-bearing and
    value-less `vue.text`.
- [`lowering_vtext.rs`](../../../../crates/vize_s1_to_s2/tests/lowering_vtext.rs)
  - element and slot-outlet `v-text` lower to `vue.text`; arg/modifier
    spellings still defer.
- [`lowering_elements.rs`](../../../../crates/vize_s1_to_s2/tests/lowering_elements.rs)
  - `v-pre` remains the unmapped-directive witness outside the dedicated
    directive-realization files.
- [`davinci_s2_text.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_text.rs)
  - S2-vs-shipped byte fixtures plus per-node patch-flag extraction across the
    realized cases above.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  and [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  - non-JS `v-text` expressions remain an explicit, source-covered refusal.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
