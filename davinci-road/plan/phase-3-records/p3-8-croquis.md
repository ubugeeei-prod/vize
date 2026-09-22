# P3-8 — Croquis registration on the non-inline plan

2026-09-23. A production `<script setup>` compile attaches a Croquis
summary. Non-inline SSR does not read reactivity facts (`PrefixScope`
consults them only when `inline` is set, and this lane refuses inline).
The plan therefore keeps the metadata binding table and emits when the
summary registers nothing that table lacks: `used_components` is empty
(the SFC drawer runs with `track_usage: false`) and every Croquis
`SetupConst` is already in the metadata. Anything else stays
`legacy.croquis`. This is the DOM lane's registration check.

A component that carries `v-slot` owns every child, including a nested
`<template v-slot>`. The walker does not open a second slot for that
template, and the vnode fallback renders no `<template>` node. A custom
directive on `<slot>` is an S2 error with no outlet op; the walker
ignores it, and so does the plan.

## Measured

`VIZE_DAVINCI_DIFFERENTIAL_CORPUS` = the checkout, 0 divergences, 0
rejected. After the two shapes below, production lanes are only `s4`
(785) and `none` (3). Compared templates are 784 of 788, with 4 legacy
compile errors skipped. The emitter sweep is 779 of 779.

`v-model` on a `v-for` or slot alias, and `v-model` with an argument on
a plain element, are errors the walker reports and then does not render.
The plan admits them and writes no attribute. A custom-directive value
the transform cannot parse is passed through raw, which is how a
disabled `v-match` / `v-when` template renders (`_ssrGetDirectiveProps`
with the authored text, plus `X_INVALID_EXPRESSION`).

TS-11's harness has no SSR surface, so the corpus gate above is the
byte-parity witness. The walker remains the fallback for option shapes
and fixtures the corpus does not compare (invalid interpolations, raw
text, inline render closures). Nothing in the checkout sweep still
selects it.

## Witnesses

- `cargo test -p vize_atelier_ssr --lib s4::`
- `cargo test -p vize_atelier_ssr --test ssr_snapshot`
- `VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --test davinci_ssr_corpus --features davinci-differential`
