# P2-11 Installment 60 - CreateSlots Patch Sites

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5405](https://github.com/ubugeeei-prod/vize/pull/5405), merged
> 2026-08-30 at `778d7969d`.

This installment pins per-node patch sites for `createSlots`. Conditional,
looped, dynamic-name, text-slot and unwrapped wrapper cases now compare the S2
DOM patch-site list against the shipped DOM lane.

The durable witnesses are:

- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - extracts shipped and S2 `createSlots` patch sites and compares them per
    node.
- [`consumer-migration-surfaces.md`](../../consumer-migration-surfaces.md)
  - records the updated witness inventory.

This installment does not tick P2-11. The latest named patch-site witness is
pinned; hydrated corpus evidence and the production-lane switch remain open.
