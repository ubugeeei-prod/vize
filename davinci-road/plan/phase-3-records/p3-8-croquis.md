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
rejected:

| sweep | result |
| --- | --- |
| emitter | 779 of 779 plan-emitted |
| production `compile_sfc` | 783 of 784 (`legacy.binding` 1, `legacy.expression_or_encoding` 1, `none` 3) |

`legacy.croquis` is empty. The binding row is `v-model` on a `v-for`
alias plus `v-model:value` (the walker drops both as errors). The
expression row is `v-match` / `v-when`. The walker stays for those, for
option shapes the plan refuses, and for the parity oracle. P3-8 stays
open: the Real Project Matrix has no SSR surface, and charter #26 still
wants the walker deleted only once nothing compared still needs it.

## Witnesses

- `cargo test -p vize_atelier_ssr --lib s4::`
- `cargo test -p vize_atelier_ssr --test ssr_snapshot`
- `VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --test davinci_ssr_corpus --features davinci-differential`
