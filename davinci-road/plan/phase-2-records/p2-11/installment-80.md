# P2-11 Installment 80 - Residual DOM Corpus Gaps

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5582](https://github.com/ubugeeei-prod/vize/pull/5582), merged
> 2026-09-01 at `b6c6948a32`.

This installment aligns remaining S2 DOM hoist behavior for conditional
`v-for` branches and nested slot carriers, keeps `v-once` runtime directives
resolved without wrapping cache initializers, and advances skipped component
slot walks so later slots keep shipped render order.

The durable witnesses are:

- [`davinci_s2_once.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_once.rs)
  - pins `v-once` directive handling.
- [`emit_vslots.rs`](../../../../crates/vize_s1_to_s2/tests/emit_vslots.rs)
  - covers slot-carrier emission order.

This installment does not tick P2-11. The production-lane switch remains open.
