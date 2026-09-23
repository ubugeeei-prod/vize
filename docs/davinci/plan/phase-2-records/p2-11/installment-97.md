# P2-11 Installment 97 - Cached Event Handlers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5643](https://github.com/ubugeeei-prod/vize/pull/5643), merged
> 2026-09-04 at `8bd1bf72c`.

This installment lands `cache_handlers`, the last DOM-relevant
`CodegenOptions` field the S2 emitter had not honored. Handler values,
modifier wrappers and patch-flag dynamic-prop decisions now follow the shipped
rule: cache when the option is enabled in the current scope, but suppress it
for setup-const handlers in the inline shape and for scoped handler params.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_cache_handlers.rs` compares cached,
uncached, scoped and binding-aware handler shapes against the shipped lane.
The S2 emit options, event wrapper, model and merged-props paths all consume
the same cache decision.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
