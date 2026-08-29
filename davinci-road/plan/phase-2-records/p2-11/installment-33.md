# P2-11 Installment 33 - V-html Raw HTML Props

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5200](https://github.com/ubugeeei-prod/vize/pull/5200), merged
> 2026-08-29 at `13cff4d99`.
> Issue: [#5199](https://github.com/ubugeeei-prod/vize/issues/5199).

This installment gives `v-html` a typed S2 representation and realizes it in
the S2 DOM emitter without flipping the shipped compiler lane. The lowering
admits element and slot-outlet `v-html` as `vue.html`, including value-less
spellings so DOM emit can preserve the shipped `innerHTML: undefined` shape.
Argument and modifier spellings still produce `defer.v-html` info diagnostics
and keep the owner fragment.

The realized cases are:

- Native elements with empty, static-child, interpolation-child, `v-if`, and
  `v-for` bodies.
- Value-bearing and value-less `v-html`.
- Static-name components and `<slot>` outlets.
- `v-bind` object spread, `:id`, dynamic `:style`, custom directives, and
  native `v-model` combined with `v-html`.
- Non-JS expressions as a source-covered `html_expression_not_js` refusal.

The durable witnesses are:

- [`folio_html.rs`](../../../../crates/vize_s2/tests/folio_html.rs)
  - exact Folio parse/print and owned-mirror coverage for value-bearing and
    value-less `vue.html`.
- [`lowering_html.rs`](../../../../crates/vize_ricalco/tests/lowering_html.rs)
  - element and slot-outlet `v-html` lower to `vue.html`; arg/modifier
    spellings still defer.
- [`lowering_elements.rs`](../../../../crates/vize_ricalco/tests/lowering_elements.rs)
  - `v-pre` remains the unmapped-directive witness outside the dedicated
    directive-realization files.
- [`davinci_s2_html.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_html.rs)
  - S2-vs-shipped byte fixtures plus per-node patch-flag extraction across the
    realized cases above.
- [`emit_unsupported_census.rs`](../../../../crates/vize_ricalco/tests/emit_unsupported_census.rs)
  and [`emit_unsupported_catalogue.rs`](../../../../crates/vize_ricalco/tests/emit_unsupported_catalogue.rs)
  - non-JS `v-html` expressions remain an explicit, source-covered refusal.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
