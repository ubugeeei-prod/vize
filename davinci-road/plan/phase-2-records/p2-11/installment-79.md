# P2-11 Installment 79 - Next DOM Corpus Residuals

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5576](https://github.com/ubugeeei-prod/vize/pull/5576), merged
> 2026-09-01 at `185d49ba9f`.

This installment reserves direct static slot prop hoists before direct static
slot vnodes, preserves forwarded slot flags with dynamic names, keeps `v-for`
direct-text props inline and narrows `createSlots`/`vShow` helper ordering.

The durable witnesses are:

- [`davinci_s2_format_residuals.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_format_residuals.rs)
  - captures reduced format residuals.
- [`emit_outlets.rs`](../../../../crates/vize_s1_to_s2/tests/emit_outlets.rs)
  - pins slot outlet emission.

This installment does not tick P2-11. The production-lane switch remains open.
