# P2-11 Installment 50 - Component Once Wrappers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5386](https://github.com/ubugeeei-prod/vize/pull/5386), merged
> 2026-08-30 at `2c5465d94`.

This installment emits component `v-once` wrappers from S2. Component once
carriers now use the shipped cache wrapper shape while preserving component
resolution, prop and child emission around the wrapper.

The durable witnesses are:

- [`davinci_s2_once.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_once.rs)
  - keeps component `v-once` output byte-identical to the shipped lane.
- [`once.rs`](../../../../crates/vize_s1_to_s2/src/emit/once.rs)
  - owns the cache wrapper emission.
- [`component.rs`](../../../../crates/vize_s1_to_s2/src/emit/component.rs)
  - threads component emission into the once wrapper path.

This installment does not tick P2-11. Component once output is covered; the
hydrated corpus evidence and production-lane switch remain open.
