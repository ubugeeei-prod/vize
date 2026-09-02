# P2-11 Installment 72 - Component Static Props Inline

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5565](https://github.com/ubugeeei-prod/vize/pull/5565), merged
> 2026-09-01 at `07ac91d602`.

This installment keeps component-root `v-slot` carrier props inline, stops
unused branch prop hoists inside `v-for` loops, and prevents all-static
component bind props from taking the broad slot-hoist path. Corpus-derived Ant
Design, Arco and Buefy cases pin the behavior.

The durable witnesses are:

- [`davinci_s2_hoist_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_hoist_order.rs)
  - compares the corpus-derived component hoist-order cases.
- [`davinci_s2_template_wrapper_component_props.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_template_wrapper_component_props.rs)
  - keeps wrapper-carried component props inline.

This installment does not tick P2-11. The production-lane switch remains open.
