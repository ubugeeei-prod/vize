# P2-11 Installment 5 — mixed text siblings (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4652](https://github.com/ubugeeei-prod/vize/pull/4652) —
`9555f1816938f8db7bdcad4513cd146ecfdf4686` on `origin/main`.

## What landed

Mixed element+text siblings emit `_createTextVNode`, including the
Vue single-space `()` convention. Dual-run battery covers
interp-then-span, static-text-then-span, and space-between-spans.

Stacked on #4651 (interpolation emit).

## Named remainder after this increment

Bindings, events, `v-if` / `v-for`, components, filters, and slots
are still later installments. The old lane stays the shipped path.
