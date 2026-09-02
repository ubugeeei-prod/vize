# P2-11 Installment 75 - Codegen Directive Helper Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5568](https://github.com/ubugeeei-prod/vize/pull/5568), merged
> 2026-09-01 at `86f40b34b0`.

This installment preserves final rank-2 helper order for codegen-only
`withDirectives` cases. It adds parity coverage for a `createSlots` default
versus dynamic-slot `v-show` ordering case.

The durable witnesses are:

- [`emit_dirs.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dirs.rs)
  - pins the codegen-only directive helper order.
- [`davinci_s2_helper_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_helper_order.rs)
  - compares the shipped helper preamble surface.

This installment does not tick P2-11. The production-lane switch remains open.
