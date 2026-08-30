# P2-11 Installment 49 - Keyed Slot Template Forwarding

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5381](https://github.com/ubugeeei-prod/vize/pull/5381), merged
> 2026-08-30 at `b408b67c8`.

This installment admits keyed slot template forwarding through the S2 DOM lane.
Forwarded slot templates keep the shipped slot table and fallback shape instead
of being rejected when the slot carrier also has a key.

The durable witnesses are:

- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - compares keyed forwarding against the shipped DOM output.
- [`emit_create_slots.rs`](../../../../crates/vize_s1_to_s2/tests/emit_create_slots.rs)
  - pins the direct `createSlots` output.
- [`slots.rs`](../../../../crates/vize_s1_to_s2/src/emit/slots.rs)
  - owns the slot table emission path.

This installment does not tick P2-11. It closes keyed forwarding, while the
hydrated corpus evidence and production-lane switch remain open.
