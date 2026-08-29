# P2-11 Installment 38 - Object V-on Modifiers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5210](https://github.com/ubugeeei-prod/vize/pull/5210), merged
> 2026-08-29 at `f3959e7e3`.
> Issue: [#5209](https://github.com/ubugeeei-prod/vize/issues/5209).

This installment realizes object-form `v-on` spreads that carry option
modifiers. The shipped DOM lane ignores modifiers on object `v-on`: the handler
object still flows through `_toHandlers(expr, true)`, including inside
`mergeProps` segments, and no individual event key is rewritten.

The durable witnesses are:

- [`merge.rs`](../../../../crates/vize_s1_to_s2/src/emit/merge.rs)
  - object `v-on` admission validates only the handler expression, matching the
    shipped object-spread path.
- [`emit_on.rs`](../../../../crates/vize_s1_to_s2/tests/emit_on.rs)
  - exact pins that object `v-on` modifiers preserve the same handler object
    and output as the modifier-free source.
- [`davinci_s2_von.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_von.rs)
  - S2-vs-shipped byte fixtures for lone, merged, and component object `v-on`
    modifiers.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  - the source-level `ObjectOnHasModifiers` witness retires from the committed
    refusal census.
- [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  - `ObjectOnHasModifiers` remains accounted for as a retired bucket.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
