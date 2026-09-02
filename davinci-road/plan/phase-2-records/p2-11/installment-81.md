# P2-11 Installment 81 - DOM Corpus Residual Parity

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5583](https://github.com/ubugeeei-prod/vize/pull/5583), merged
> 2026-09-01 at `e65a078d37`.

This installment closes residual corpus divergences left by the post-#5582
matrix run. It adds S2 lowering and emission parity for `v-pre` inert text,
template-if namespace/block shape, slot outlet style normalization, helper
ordering and `v-for` child hoists.

The durable witnesses are:

- [`davinci_s2_v_pre.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_v_pre.rs)
  - pins inert text inside `v-pre`.
- [`lowering_elements.rs`](../../../../crates/vize_s1_to_s2/tests/lowering_elements.rs)
  - covers namespace and template-if lowering.

This installment does not tick P2-11. The production-lane switch remains open.
