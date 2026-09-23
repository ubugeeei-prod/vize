# P3-8 — Fourteenth slice: non-form v-model, v-pre, slot directives

2026-09-23. The plan owns three more shapes only where the legacy walker
already emits those bytes. Nothing was rewritten to force a shape onto the
plan. The legacy walker stays.

## Owned

- **Non-form `v-model` with no argument** (`div`, `svg`, `math`, `template`).
  `process_v_model_on_element` emits no attribute off `input` / `textarea` /
  `select`. An argument (`<input v-model:foo>`) stays `legacy.binding`.
- **`v-pre`** (`drop.v-pre`, `lower.v-pre-text`). The directive is dropped and
  the frozen subtree is ordinary text, including mustaches and the whitespace
  inside `<pre>`. `<slot v-pre>` still carries an S2 diagnostic, so it stays
  `surface_semantics`.
- **`<slot>` directives other than `v-bind`.** The legacy outlet walker reads
  only attributes and `v-bind`. `v-html`, `v-text`, and `v-cloak` on an outlet
  now emit from the plan. `v-show`, `v-model`, `v-once`, and `v-memo` on
  `<slot>` still fail the S2 diagnostic gate before emission.

## Still refused (checkout smoke)

Emitter sweep, `VIZE_DAVINCI_DIFFERENTIAL_CORPUS` = the checkout, 0
divergences, 0 rejected: **764 of 779** compared templates emit from the plan
(was 750 of 779 before this slice). Legacy reasons:

| reason              | count | shapes                                                                                                                                                                                            |
| ------------------- | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `surface_semantics` |     9 | `@vize:` comments (legacy renders them; S2 drops them), `<slot>` with `v-show` / `v-model` / `v-once` / `v-memo` / `v-pre`, dropped attributes on structural `<template>` (an S2 info diagnostic) |
| `binding`           |     3 | non-identifier dynamic key (`:[row.field]`), dynamic `v-model` argument, `<Suspense>` default slot                                                                                                |
| `operation`         |     3 | non-identifier dynamic slot names (`#[names.current]`, template-literal names)                                                                                                                    |

Production adapter sweep, same checkout, 0 divergences: **179 of 784**
compared (`legacy.croquis` 597, `surface_semantics` 6, `operation` 2,
`binding` 1, `none` 3). Croquis summaries and the Real Project Matrix TS-11
witness are still open, so the walker is not deleted and P3-8 stays open.

## Witnesses

- `cargo test -p vize_atelier_ssr --lib s4::`
- `cargo test -p vize_atelier_ssr --test ssr_snapshot`
- `VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --test davinci_ssr_corpus --features davinci-differential`
