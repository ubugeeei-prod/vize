# P2-11 Installment 57 - V-once Patch Flag Sites

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5400](https://github.com/ubugeeei-prod/vize/pull/5400), merged
> 2026-08-30 at `6c21f0a52`.

This installment pins per-node patch sites for `v-once`. Once output had direct
byte witnesses; this records the exact patch-site list so cache-wrapper changes
cannot hide a flag-program drift.

The durable witnesses are:

- [`davinci_s2_once.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_once.rs)
  - compares S2 and shipped DOM output and extracts the once patch sites.
- [`consumer-migration-surfaces.md`](../../consumer-migration-surfaces.md)
  - records the updated witness inventory.

This installment does not tick P2-11. `v-once` patch sites are pinned; the
hydrated corpus evidence and production-lane switch remain open.
