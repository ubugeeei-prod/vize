# P2-11 Installment 4 — interpolations (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4651](https://github.com/ubugeeei-prod/vize/pull/4651) —
`e859e71eeae88d873271b2b5840e2ec8edd2524f` on `origin/main`.

## What landed

Js interpolations and mixed text+interpolation compounds compile from
`TextFacts`, never from the opaque rebuilt source (pessimal law 5).
Matches the shipped lane byte-for-byte, including `_toDisplayString`,
TEXT patch flags, and omitting TEXT when root static props are
hoisted.

Dual-run battery covers root / element / nested / compound
interpolations.

## Named remainder after this increment

Mixed element+text siblings stay `Unsupported` until
`createTextVNode` (installment 5). Filters (`ExprRef::Filter`),
bindings, and components remain refused.
