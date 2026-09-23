# P3-8 — Fifteenth slice: ignored slot directives

2026-09-23. `v-show`, `v-once`, and `v-memo` on `<slot>` are Info
deferrals. The legacy SSR walker does not render them, and the plan
now emits that same outlet. `v-pre` still freezes the fallback as
text (`{{ not }}` stays literal), so it stays `surface_semantics`.

## Still refused (checkout smoke)

Emitter sweep, `VIZE_DAVINCI_DIFFERENTIAL_CORPUS` = the checkout, 0
divergences, 0 rejected: **767 of 779** compared templates emit from
the plan (was 764 of 779). Legacy reasons:

| reason              | count |
| ------------------- | ----: |
| `surface_semantics` |     6 |
| `binding`           |     3 |
| `operation`         |     3 |

Production adapter sweep, same checkout, 0 divergences: **182 of 784**
compared (`legacy.croquis` 597, `surface_semantics` 3, `operation` 2,
`binding` 1, `none` 3). The walker stays.

## Witnesses

- `cargo test -p vize_atelier_ssr --lib s4::`
- `cargo test -p vize_atelier_ssr --test ssr_snapshot`
- `VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --test davinci_ssr_corpus --features davinci-differential`
