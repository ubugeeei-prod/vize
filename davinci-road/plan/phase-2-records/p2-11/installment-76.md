# P2-11 Installment 76 - Static Prop Hoist Surfaces

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5569](https://github.com/ubugeeei-prod/vize/pull/5569), merged
> 2026-09-01 at `66fbe814ba`.

This installment aligns static prop hoist surfaces for foreign built-in
components, namespace-crossing fragment roots and slot-forwarding components.
It preserves shipped numbering for SVG `TransitionGroup` output and legacy
global-constant prop hoists.

The durable witnesses are:

- [`emit_static_hoist.rs`](../../../../crates/vize_s1_to_s2/tests/emit_static_hoist.rs)
  - pins static prop hoist behavior.
- [`davinci_s2_static_vnode_hoist.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_static_vnode_hoist.rs)
  - compares the shipped static-vnode hoist surface.

This installment does not tick P2-11. The production-lane switch remains open.
