# P2-11 Installment 82 - Final DOM Corpus Residuals

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5585](https://github.com/ubugeeei-prod/vize/pull/5585), merged
> 2026-09-01 at `7fad0210b5`.

This installment aligns S2 DOM emission with the shipped lane for the last
corpus residuals after #5583. It preserves patchless top-level `NaN` and
`Infinity` props while keeping nested global constants dynamic, and matches
legacy `Transition` slot prop hoisting, foreign SVG hoist splitting and valued
`v-else` branch conditions.

The durable witnesses are:

- [`davinci_s2_corpus_residuals.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_corpus_residuals.rs)
  - pins the final reduced residual snippets.
- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  - preserves patch flag behavior for the same surfaces.

This installment does not tick P2-11. The production-lane switch remains open.
