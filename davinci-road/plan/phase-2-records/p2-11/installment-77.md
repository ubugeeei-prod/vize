# P2-11 Installment 77 - Slot Branch Key Reservations

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5572](https://github.com/ubugeeei-prod/vize/pull/5572), merged
> 2026-09-01 at `d4f6d75936`.

This installment keeps conditional slot template keys from shifting branch-key
allocation in the parent slot context. Key accounting now counts only branch
keys emitted inside the slot function, matching Koel and Nuxt UI fixture output.

The durable witnesses are:

- [`emit_create_slots.rs`](../../../../crates/vize_s1_to_s2/tests/emit_create_slots.rs)
  - pins nested conditional slot branch keys.
- [`vif_pass_keys.rs`](../../../../crates/vize_s1_to_s2/tests/vif_pass_keys.rs)
  - covers the branch-key reservation law.

This installment does not tick P2-11. The production-lane switch remains open.
