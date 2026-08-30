# P2-11 Installment 59 - Slot Outlet Patch Sites

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5404](https://github.com/ubugeeei-prod/vize/pull/5404), merged
> 2026-08-30 at `86e52e3c7`.

This installment pins per-node patch sites for slot outlets. Named, dynamic,
forwarded and scoped outlet cases now compare not only bytes but the exact flag
sites the shipped DOM lane produces.

The durable witnesses are:

- [`davinci_s2_outlets.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_outlets.rs)
  - extracts shipped and S2 slot outlet patch sites and compares them per node.
- [`consumer-migration-surfaces.md`](../../consumer-migration-surfaces.md)
  - records the updated witness inventory.

This installment does not tick P2-11. Slot outlet patch sites are pinned; the
hydrated corpus evidence and production-lane switch remain open.
