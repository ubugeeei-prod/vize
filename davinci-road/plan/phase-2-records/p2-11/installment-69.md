# P2-11 Installment 69 - Scoped Slot Key Prop Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5562](https://github.com/ubugeeei-prod/vize/pull/5562), merged
> 2026-09-01 at `3145454f43`.

This installment keeps scoped-slot all-static-bind component props from moving
ahead of nested keyed component props. The Ant Design Vue context-menu case is
recorded as a hoist-order regression so S2 DOM output preserves the shipped
lane's prop order.

The durable witnesses are:

- [`davinci_s2_hoist_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_hoist_order.rs)
  - pins the scoped-slot component prop order against shipped output.
- [`component.rs`](../../../../crates/vize_s1_to_s2/src/emit/component.rs)
  - owns the inline-vs-hoisted component prop decision.

This installment does not tick P2-11. The production-lane switch remains open.
