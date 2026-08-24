# P2-11 Installment 9 — `v-on` modifiers (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4673](https://github.com/ubugeeei-prod/vize/pull/4673) —
`081db167da0621afa05daa9a6b1dfc236890e0e5` on `origin/main`.

## What landed

Static-name `v-on` event / key / option modifiers match the shipped
DOM lane: `_withModifiers` / `_withKeys`,
`onClickCapture` / `Once` / `Passive` suffixes, `@click.right` →
`onContextmenu`, `@click.middle` → `onMouseup`. Hydration stays
`onClick`-only unless `withKeys` is used.

Dual-run includes native `v-if` cases already on main.

## Named remainder after this increment

`.native`, object-spread `v-on`, and duplicate handlers stay
`Unsupported`.
