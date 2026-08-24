# P2-11 Installment 7 — static-name `v-on` (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4667](https://github.com/ubugeeei-prod/vize/pull/4667) —
`05dbdfe73cc56e944acab4bd449c16c7200ee09a` on `origin/main`.

## What landed

Static-name `v-on` as `onClick` / `onKeyup` props with the shipped
patch flags (`PROPS`, plus `NEED_HYDRATION` for `keyup`). Inline
handlers wrap as `$event => (expr)` to match the shipped DOM lane.

#4654 / #4655 / #4658 were squash-merged into a parent feature
branch by accident. They are **not** `origin/main` installment SHAs.
This PR targeted `main` after v-bind (#4657).

## Named remainder after this increment

Modifiers, object-spread `v-on`, and duplicate handlers stay
`Unsupported`. Dynamic event names and names containing `:` also
refuse.
