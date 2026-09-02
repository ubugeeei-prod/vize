# P2-11 Installment 78 - Modifier Helper Preamble Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5573](https://github.com/ubugeeei-prod/vize/pull/5573), merged
> 2026-09-01 at `466be4eeac`.

This installment orders same-rank modifier helpers by final generated call
position after preferred helpers are handled. It keeps directive, model and
event modifier helper preambles aligned with the shipped DOM lane.

The durable witnesses are:

- [`emit_dirs.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dirs.rs)
  - covers the directive/model/event modifier helper mix.
- [`buf.rs`](../../../../crates/vize_s1_to_s2/src/emit/buf.rs)
  - owns same-rank helper preamble ordering.

This installment does not tick P2-11. The production-lane switch remains open.
