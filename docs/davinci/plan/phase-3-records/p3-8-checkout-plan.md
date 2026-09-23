# P3-8 — Sixteenth slice: checkout templates emit from the plan

2026-09-23. The checkout emitter sweep is **779 of 779** plan-emitted,
0 divergences, 0 rejected. The shapes that were still on the legacy
walker now match its bytes:

- dynamic slot names (`names.current`, `` `filter-${col}` ``, `names[0]`)
- dynamic `v-bind` keys (`:[a+b]`, `:[row.field]`)
- dynamic `v-model` arguments, without a modifiers object
- `@vize:` comments kept in the tree and written into the HTML string
- `v-model` on `<slot>`, which the walker ignores
- empty `<slot v-pre>` (a fallback mustache still stays legacy)
- attributes dropped from an unwrapped `<template>`
- `<Suspense><template #default>` renders no `<template>` tag

`v-pre` with a fallback mustache still freezes text, so that fixture
stays refused. Production `compile_sfc` is **185 of 784** (`legacy.croquis`
597, `operation` 2, `surface_semantics` 1). The walker stays, and P3-8
stays open.

## Witnesses

- `cargo test -p vize_atelier_ssr --lib s4::`
- `cargo test -p vize_atelier_ssr --test ssr_snapshot`
- `VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --test davinci_ssr_corpus --features davinci-differential`
