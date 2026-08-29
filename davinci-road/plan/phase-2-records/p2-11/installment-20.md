# P2-11 Installment 20 — dynamic `v-bind` keys and modifiers (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4811](https://github.com/ubugeeei-prod/vize/pull/4811) moved
dynamic-argument `v-bind` keys and their `.camel`, `.prop` and `.attr`
modifier forms out of the S2 DOM emitter's local refusal surface. The merge
commit is `72fff08c7c43550b6e7d2d341f00bb4b3922d5d7`.

The emitter now writes computed object keys for native elements, components,
`mergeProps`, conditional branches, loops and slot outlets. Runtime camelizing
uses the shipped `_camelize` helper; `.prop` and `.attr` prefix the computed
key in Vue's modifier order. Dynamic keys keep `FULL_PROPS`, hydration and
keyed-fragment behavior byte-identical to the shipped lane.

The durable current witnesses are:

- [`emit_dynamic_bind_keys.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dynamic_bind_keys.rs)
  — direct S2 emission snapshots for computed keys, modifier composition,
  `mergeProps`, `v-if`, `v-for` and slot outlets.
- [`davinci_s2_dynamic_bind_keys.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_dynamic_bind_keys.rs)
  — a 14-fixture S2-vs-shipped byte-for-byte battery over those families.

This installment does not tick P2-11. Filters and local slot/outlet guard-only
shapes remain; the old DOM lane is still the production path, and the
task-level corpus, patch-flag, allocation-budget and publish-graph gates remain
open.
