# P2-9 Installment 8 — Vue 2 atelier comparator (2026-08-24)

> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

Installment 7 landed the sugar (ops, admission, `legacy-sugar` pass)
and recorded two honest gaps: no Vue 2 atelier dual-run, and no P1-9
residual re-measure. This installment closes the first.

## What landed

A second comparator target, `davinci_s2_transform_vue2`, compiled only
with `--features legacy` (`[[test]] required-features`). The shipped
`.sync` / `slot-scope` expansion lives behind that feature; a Vue 2
dual-run without it would compare S2's rewrite against an inert Vue 3
tree. The default workspace suite still pins the Vue 3 battery.
`clippy-and-test` invokes the new target on the existing Test step
line (`check.yml` line count unchanged).

`compare_with` carries the dialect. Vue 3 keeps calling `compare`
(default `VueVersion::V3`). Vue 2 passes `VueVersion::V2` into both
lanes: the shipped transform via `TransformOptions.dialect`, and S2
via `LegacyCaps::for_version` at `lower_with_caps` (the pass table
then follows `lowered.caps`, `walks=7`).

## Battery (exact-pinned)

Six templates, zero divergence, counters pinned in the witness:

| name               | what it pins                                                  |
| ------------------ | ------------------------------------------------------------- |
| `sync`             | `:title.sync` reconstructs as a component model on both lanes |
| `sync-camel`       | leftover `.camel` does not leak onto the reconstructed model  |
| `slot-scope`       | default `slot-scope` groups as invented slot content          |
| `named-slot-scope` | named `slot` + `slot-scope`                                   |
| `if`               | sugar-free `v-if` still agrees under the 7-walk Vue 2 table   |
| `native-keycode`   | `.native` strip + numeric keyCode rewrite                     |

`.sync` legalize emits bind + `update:` listener. The legacy collector
already folds that span-shared product into `PModel`. S2's surface
collector now folds the same pair when the handler text is the
legalize product (`$event => ((value) = $event)`), so an explicit
user-authored `@update:` listener is not swallowed. `.camel` rides a
stub product bind on the legacy side that S2 does not emit; the fold
drops leftover bind modifiers to match.

## Named gap, still open

Interpolation filters (`{{ msg | cap }}`) still diverge at the text
projection: the legacy collector reports the authored `msg | cap`
spelling, S2 legalize rewrites the interpolation to `_filter_cap(msg)`
before text facts are published. Mixed text-runs that absorbed a pipe
into a compound opaque are the same family (installment 7). Not in
this battery; not silently skipped.

P1-9 residual re-measure is still not this installment: wrapping
happens on S2 `ExprRef`s after lowering and does not feed
`transform_expression/reparse.rs`. Inventing a 12.73% figure without
running the counters would violate the task. The series box in
`phase-2.md` stays open.

## House

Every touched file ≤ 350 (largest new: `s2_support/compare.rs` 224).
`s2_support` stays a test module (no `src/**/mod.rs`). ricalco
untouched, so `no_std + alloc` is unchanged. Assertions are
exact-equality on the pinned `Counters` (TS-13).
