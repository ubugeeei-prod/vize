# P2-11 Installment 70 - Transition Prop Hoist Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5563](https://github.com/ubugeeei-prod/vize/pull/5563), merged
> 2026-09-01 at `b06a3edc65`.

This installment keeps `Transition` and `transition` static props inline unless
the slot outlet is direct. It prevents forwarded-slot fallback content from
moving ahead of the shipped lane's prop order.

The durable witnesses are:

- [`davinci_s2_hoist_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_hoist_order.rs)
  - covers transition props with forwarded slots and fallback content.
- [`call_props.rs`](../../../../crates/vize_s1_to_s2/src/emit/component/call_props.rs)
  - carries the slot-outlet-sensitive component prop path.

This installment does not tick P2-11. The production-lane switch remains open.
