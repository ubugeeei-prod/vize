# P2-11 Installment 73 - Helper Preamble Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5566](https://github.com/ubugeeei-prod/vize/pull/5566), merged
> 2026-09-01 at `6f21c0432e`.

This installment preserves the shipped Vue helper preamble order for
normalize/modifier helper ranks by using final alias occurrence order. Other
helper ranks keep the existing preferred/helper registration fallback.

The durable witnesses are:

- [`davinci_s2_helper_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_helper_order.rs)
  - pins corpus-derived DOM helper-order regressions.
- [`buf.rs`](../../../../crates/vize_s1_to_s2/src/emit/buf.rs)
  - owns helper preamble ordering.

This installment does not tick P2-11. The production-lane switch remains open.
