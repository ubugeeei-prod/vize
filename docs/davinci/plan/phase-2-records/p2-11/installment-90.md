# P2-11 Installment 90 - Inline Root Prop Hoist

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5634](https://github.com/ubugeeei-prod/vize/pull/5634), merged
> 2026-09-04 at `d6552b53d`.

This installment lets inline S2 DOM emit hoist root props the same way the
shipped lane does when `<script setup>` puts `render` inside `setup()`.
The component emit was split into a smaller entry path, and static prop
emission gained the root-hoist arm that only exists for the inline shape.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_inline_root_hoist.rs` and
`crates/vize_atelier_dom/tests/davinci_s2_inline_root_hoist_components.rs`
compare the S2 output against the shipped lane for native and component roots.
The support binding fixtures keep the inline read shape explicit.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
