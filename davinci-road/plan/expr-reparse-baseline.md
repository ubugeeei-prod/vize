# Expression re-parse baseline (P0-3)

> [!NOTE]
> Generated from the P0-3 bench recorders. Each number is the delta of
> `vize_atelier_core::expr_parse_probe::expr_parse_count()` around **one**
> fused compile of the fixture — the count of oxc expression parses (one per
> parse-scoped `oxc_allocator::Allocator` creation) on that backend's compile
> path today. The probe instruments 18 sites: 16 in `vize_atelier_core`
> (transform + codegen stages) and 2 in `vize_atelier_vapor`
> (`generate/expression.rs`). Counts are deterministic — identical across
> every recorded run.
>
> This is the number P1 drives toward "parse once": after P1-7 retains
> expression ASTs, TS-26 asserts that
> `davinci.expr.parses == distinct expressions`. Regenerate by running
> `cargo bench -p vize_atelier_dom -p vize_atelier_vapor -p vize_atelier_ssr`
> and reading the `davinci.expr.parses <backend> <fixture> <count>` stderr
> lines.

## Counts per fused compile (2026-08-13, probe at P0-3 introduction)

| fixture       | dom | vapor |     ssr |
| ------------- | --: | ----: | ------: |
| small         |   1 |     0 |       4 |
| medium        |  33 |     4 |      37 |
| large         | 106 |    45 |     155 |
| stress-deep   |  24 |    16 |      48 |
| stress-wide   | 102 |     0 |     100 |
| stress-interp |   0 |     0 | **500** |

## What the numbers say

- **SSR re-parses every interpolation**: `stress-interp` holds 500
  interpolations and the SSR path parses expression text 500 times where the
  DOM and Vapor paths parse none. The SSR codegen stage resolves
  interpolation expressions from source text each time.
- **Attribute bindings cost a parse each on DOM and SSR**: `stress-wide`
  carries 50 `:bound` attributes + 50 `@event` handlers and lands at ~100
  parses on both backends — roughly one parse per dynamic attribute — while
  Vapor's generate path resolves them without oxc.
- **The same file pays differently per backend** (`large`: dom 106 /
  vapor 45 / ssr 155): every backend re-derives expression facts from text
  independently, which is fault line 1 of
  [motivation.md](../motivation.md) in number form.

## Scope and caveats

- Compile path only. `vize_croquis`'s analysis-side parses (13 further oxc
  parse sites) are not instrumented in this round; the croquis analyze
  benches (P0-2) price that cost in wall/allocs instead.
- The probe counts parse-scoped arena creations, so a site that parses twice
  behind one arena would undercount — no such site exists in the
  instrumented set as of this baseline.
- Counter and sites are temporary: deleted when phase 1 lands retained
  expressions (see `expr_parse_probe`'s module docs).
