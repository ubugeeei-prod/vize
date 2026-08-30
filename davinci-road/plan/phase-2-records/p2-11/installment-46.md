# P2-11 Installment 46 - If Branches Containing V-for

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5376](https://github.com/ubugeeei-prod/vize/pull/5376), merged
> 2026-08-30 at `eef9f2064`.

This installment emits `v-if` branch regions whose selected branch contains a
`v-for`. The S2 DOM emitter now keeps the shipped fragment and render-list
shape instead of refusing nested structural combinations at the branch boundary.

The durable witnesses are:

- [`davinci_s2_if_for.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_if_for.rs)
  - compares the nested `v-if` / `v-for` S2 output against the shipped DOM lane.
- [`emit_if.rs`](../../../../crates/vize_s1_to_s2/tests/emit_if.rs)
  - pins the direct S2 emitter output for the structural lowering.
- [`vif.rs`](../../../../crates/vize_s1_to_s2/src/emit/vif.rs)
  - owns the branch emission path that this installment widens.

This installment does not tick P2-11. It closes this structural emit class, but
the hydrated corpus evidence and production-lane switch remain open.
