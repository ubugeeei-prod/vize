# P2-11 Installment 98 - Printed Cache Slot Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5649](https://github.com/ubugeeei-prod/vize/pull/5649), merged
> 2026-09-04 at `74c64d33d`.

This installment numbers cache slots in printed order. Earlier S2 emit paths
reserved cache ids from discovery order, which can differ from the order the
render body finally prints after hoists, slots, `v-once`, `v-memo` and model
keys are assembled.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_cache_slot_order.rs` pins the cache
slot numbering across handlers, hoists, slots, memo, once and model-key paths.
`emit::cache_slots` owns the printed-order numbering API used by those emit
sites.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
