# P2-11 Installment 11 — object-spread `v-bind` (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4678](https://github.com/ubugeeei-prod/vize/pull/4678) —
`d15ce229b1727db4d582e795574ee2caeea302b6` on `origin/main`.

## What landed

Nameless `v-bind="obj"` emits `_normalizeProps(_guardReactiveProps(obj))`
when it is the only props source, and `_mergeProps(...)` in authored
order when it sits beside attrs / named binds / events / `v-if` keys.

Pinned shipped-lane quirks, with exact-equality tests: lone duplicate
object binds keep the first; `FULL_PROPS` without CLASS/STYLE/PROPS
bits; skip `normalizeClass` / `normalizeStyle` inside merge objects.

## Named remainder after this increment

`v-on` object form stays `Unsupported` (`emit/merge.rs`). Object-bind
modifiers refuse. This is still the obvious next emit increment.
