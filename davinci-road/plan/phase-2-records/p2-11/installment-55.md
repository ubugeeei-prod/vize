# P2-11 Installment 55 - V-memo Patch Flag Sites

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5398](https://github.com/ubugeeei-prod/vize/pull/5398), merged
> 2026-08-30 at `db06c3aa1`.

This installment pins per-node patch sites for `v-memo`. The S2 DOM lane already
had byte witnesses for memo output; this adds an exact patch-site witness so the
flag program cannot drift while preserving one small rendered fixture.

The durable witnesses are:

- [`davinci_s2_memo.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_memo.rs)
  - compares S2 and shipped DOM output and extracts the memo patch sites.
- [`consumer-migration-surfaces.md`](../../consumer-migration-surfaces.md)
  - records the updated witness inventory.

This installment does not tick P2-11. `v-memo` patch sites are pinned; the
hydrated corpus evidence and production-lane switch remain open.
