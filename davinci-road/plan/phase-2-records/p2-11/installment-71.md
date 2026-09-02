# P2-11 Installment 71 - Nested Component Prop Hoist Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5564](https://github.com/ubugeeei-prod/vize/pull/5564), merged
> 2026-09-01 at `eada2aa7dd`.

This installment keeps parent all-static component prop objects inline when
nested component `v-for` props must stay before them. The component static-prop
hoistability check becomes fallible so nested analysis can reuse emit context
without reordering shipped output.

The durable witnesses are:

- [`davinci_s2_hoist_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_hoist_order.rs)
  - pins the Ant Design row/column prop-order regression.
- [`call_props.rs`](../../../../crates/vize_s1_to_s2/src/emit/component/call_props.rs)
  - owns the nested component prop hoist decision.

This installment does not tick P2-11. The production-lane switch remains open.
