# P2-11 Installment 103 - S2 Option Combination Matrix

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: _pending - the number, merge date and squash SHA are filled in at
> merge, as every prior installment's line was._

This installment adds the first production-option combination matrix for the
S2 DOM emitter. The individual witnesses already cover `cache_handlers`,
`scope_id`, `prefix_identifiers`, `module`, and `inline`, but real SFC compiles
exercise several of those switches together. The matrix keeps those surfaces
byte-for-byte compared with the shipped lane so option interactions cannot
hide behind single-option parity.

The new witness exposed two combination-only gaps. In the default lane,
`v-for` aliases are stored as raw patterns rather than materialized slot params,
so `cache_handlers` missed them and cached an item handler that the shipped
lane keeps dynamic. The cache gate now reads both callback-param stacks. In the
inline lane, cached static child arrays are render-time cache entries, not
module-level hoists, so they keep runtime scoped-style handling instead of
baking the `scope_id` pair into every cached child VNode.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_option_matrix.rs` runs a 32-template
battery under three option bundles: `cache_handlers + scope_id`, non-inline
script setup module mode, and inline script setup module mode. The held-out
inline component/static-prop cases remain documented as pre-existing inline
surface residuals.

The focused gate is:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_option_matrix
```

This installment does not tick P2-11. The production-lane switch remains open,
and the old DOM lane is still the shipped non-profiled compile path.
