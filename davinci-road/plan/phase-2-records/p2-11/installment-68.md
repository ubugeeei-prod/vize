# P2-11 Installment 68 - Residual DOM Corpus Gaps

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5552](https://github.com/ubugeeei-prod/vize/pull/5552), merged
> 2026-09-01 at `af800fd399`.

This installment closes the next residual DOM corpus group after component
class binds. Template-wrapper component props, foreign component prop hoists,
static template-if keys, empty looped slot entries, `v-once` text nodes,
bounded element attr patch flags and authored padding in dynamic styles now
have focused S2-vs-shipped witnesses.

The PR's corpus probe measured the hydrated residual set at 41,580 files,
41,223 templates, 41,207 comparisons, 16 old-lane skips, zero S2 refusals and
57 remaining divergences, down from 63.

This installment does not tick P2-11. It reduced named residual buckets; the
production-lane switch remains open.
