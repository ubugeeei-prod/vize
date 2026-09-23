# P2-11 Installment 10 — native `v-for` (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4674](https://github.com/ubugeeei-prod/vize/pull/4674) —
`1cecf6b4aa3fa47fe9ec55aaf696aace43fab64a` on `origin/main`.

## What landed

Native `v-for` as `_renderList` + `_Fragment` with the shipped patch
flags (`KEYED_FRAGMENT` / `UNKEYED_FRAGMENT` / `STABLE_FRAGMENT`).
Identifier aliases only this installment. Numeric sources are the
stable-fragment arm.

#4662 (same title, targeting a parent feature branch) was closed, not
merged.

## Named remainder after this increment

Destructured aliases and `<template v-for>` stay `Unsupported`.
Component `v-for` lands later in installment 12.
