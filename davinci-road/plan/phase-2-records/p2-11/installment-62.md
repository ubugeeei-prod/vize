# P2-11 Installment 62 - Nested Interactive End Tags

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5515](https://github.com/ubugeeei-prod/vize/pull/5515), merged
> 2026-08-31 at `85fe7bf151`.

This installment preserves the recovery boundary for nested interactive content
through S1 and S1-to-S2 lowering. Direct nested anchors/buttons and descendant
end-tag fallout now emit the same DOM source as the shipped lane.

The durable witnesses are:

- [`nested_interactive_recovery.rs`](../../../../crates/vize_armature/tests/nested_interactive_recovery.rs)
  - keeps redundant recovery end tags recoverable while unrelated hard end tags
    remain hard.
- [`surface_fidelity.rs`](../../../../crates/vize_s1/tests/surface_fidelity.rs)
  - pins the S1 surface text and close-kind preservation.
- [`emit_nested_interactive.rs`](../../../../crates/vize_s1_to_s2/tests/emit_nested_interactive.rs)
  - compares direct and descendant nested interactive cases byte-for-byte
    against the shipped DOM lane.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
