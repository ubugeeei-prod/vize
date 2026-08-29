# P2-11 Installment 37 - Object V-bind Modifiers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5208](https://github.com/ubugeeei-prod/vize/pull/5208), merged
> 2026-08-29 at `4e577b62`.

This installment realizes object-form `v-bind` spreads that carry `.prop`,
`.attr` or `.camel` modifiers. The shipped DOM lane treats these modifiers as
inert for object spreads: the spread expression still flows through the same
`normalizeProps` / `guardReactiveProps` / `mergeProps` path, and no individual
prop key is rewritten.

The durable witnesses are:

- [`emit_merge.rs`](../../../../crates/vize_s1_to_s2/tests/emit_merge.rs)
  - exact pins that object `v-bind` modifiers preserve the same spread
    expression and merge shape as the modifier-free source.
- [`davinci_s2_bind_modifiers.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_bind_modifiers.rs)
  - S2-vs-shipped byte fixtures for lone and merged object `v-bind`
    modifiers.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  - the source-level `ObjectBindHasModifiers` witness retires from the
    committed refusal census.
- [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  - `ObjectBindHasModifiers` remains accounted for as a retired bucket.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
