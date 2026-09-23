# P2-11 Installment 8 — native `v-if` (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4669](https://github.com/ubugeeei-prod/vize/pull/4669) —
`b2d13f18b202260dfd320d72b66e112f9f617670` on `origin/main`.

## What landed

Native `v-if` / `v-else-if` / `v-else` as the shipped ternary
(`_openBlock` + `_createElementBlock` vs
`_createCommentVNode("v-if", true)`). Static `key` is quoted; numeric
branch keys stay unquoted. Sibling chains share one counter; nested
chains reset.

#4659 targeted a parent feature branch; this PR is the main-line
installment.

## Named remainder after this increment

Template-fragment `v-if` stays `Unsupported`. Component `v-if` was
still refused here and lands later in installment 12. Dynamic branch
keys (`BranchKeyKind::Dynamic`) and non-Js conditions refuse.
