# Pre-S2 traversal baseline (P2-12a)

> [!NOTE]
> The phase-2 "before". Each number is the delta of
> `vize_atelier_core::walk_probe` around **one** fused compile of the fixture
> on that backend — the template-node **visits** and stage tree-**walks**
> today's still-live pipeline makes. Phase 2 replaces the stage-per-tree
> pipeline with S2 and a pass manager, and its exit gate compares the result
> against this table, so the table is recorded at phase start, before the work
> that could bias it. This is the P0-3 convention
> ([`expr-reparse-baseline.md`](./expr-reparse-baseline.md)) applied to
> traversal instead of expression parses.

## Reproducing

```sh
cargo test -p vize_atelier_dom -p vize_atelier_ssr -p vize_atelier_vapor \
  --test davinci_walk_baseline -- --nocapture
```

Each backend's `tests/davinci_walk_baseline.rs` prints one
`davinci.walk <backend> <fixture> walks=<n> visits=<n> <stage>=<walks>/<visits>…`
line per fixture and then asserts the whole table at once, so a re-record
prints every row rather than stopping at the first drift. The counters are
process-global and monotone, so each file holds a single `#[test]` in its own
binary — the `davinci_expr_reparse_floor.rs` shape.

**Determinism** (the P0-2/P0-5 two-run convention): the command above was run
twice at `232870a8` and the 18 output lines were byte-identical. Walk counts,
like alloc counts and unlike wall times, are deterministic and
machine-independent — they depend on the fixture and the pipeline shape, not
on the machine — so `budgets.toml [traversal]` gates them **exactly** from day
one and does not wait for the Blacksmith reference runner.

## Counts per fused compile (2026-08-19, rev `232870a8`)

`walks` is the number of stage traversals entered; `visits` the number of
template nodes those traversals dispatched on. Every backend runs the same
transform lane, which is why the `transform` column is identical across the
three tables — a cross-check on the probe rather than a coincidence.

### DOM (`compile_template_with_options`, default options)

| fixture       | walks | visits | transform | codegen |
| ------------- | ----: | -----: | --------: | ------: |
| small         |     2 |     11 |         8 |       3 |
| medium        |     2 |     62 |        33 |      29 |
| large         |     2 |     86 |        57 |      29 |
| stress-deep   |     2 |    134 |        72 |      62 |
| stress-wide   |     2 |      3 |         2 |       1 |
| stress-interp |     2 |   1102 |      1001 |     101 |

### SSR (`compile_ssr`)

| fixture       | walks | visits | transform | ssr_codegen |
| ------------- | ----: | -----: | --------: | ----------: |
| small         |     2 |     16 |         8 |           8 |
| medium        |     2 |    118 |        33 |          85 |
| large         |     2 |    106 |        57 |          49 |
| stress-deep   |     2 |    144 |        72 |          72 |
| stress-wide   |     2 |      4 |         2 |           2 |
| stress-interp |     2 |   2002 |      1001 |        1001 |

### Vapor (`compile_vapor`, default options)

| fixture       | walks | visits | transform | vapor_lower |
| ------------- | ----: | -----: | --------: | ----------: |
| small         |     2 |     25 |         8 |          17 |
| medium        |     2 |    102 |        33 |          69 |
| large         |     2 |    127 |        57 |          70 |
| stress-deep   |     2 |    256 |        72 |         184 |
| stress-wide   |     2 |      4 |         2 |           2 |
| stress-interp |     2 |   3102 |      1001 |        2101 |

## What the numbers say

- **Every backend pays the transform lane in full, then walks the tree
  again.** `walks = 2` on every row: one transform traversal, one backend
  traversal. The transform column is byte-identical across the three tables
  because all three backends run the same lane over the same tree — the
  duplicate-work story of [motivation.md](../motivation.md), in the traversal
  dimension rather than the parse dimension.
- **Vapor lowering re-walks the same children several times.** `stress-deep`:
  72 transform visits against 184 lowering visits (2.6×); `stress-interp`:
  1001 against 2101 (2.1×). Vapor lowering has six distinct child-list walkers
  (`transform_children`, the `<template>` peel, three deferred-children loops
  and the text-run collector) and an element whose children are dynamic is
  walked by more than one of them. Region-owning `ui.for` / `ui.if` ops
  (P2-5a) and fusion (P2-2) are aimed exactly here.
- **SSR's codegen visit count tracks the transform lane's on nested input and
  exceeds it on wide input.** `stress-deep` is 72/72; `medium` is 33/85,
  because SSR descends into slot and component children through a second
  dispatcher (`vnode_child_expression`) as well as `process_child`.
- **DOM codegen is the cheapest backend traversal** (29 visits on both
  `medium` and `large` against 33 and 57 transform visits): hoisting and the
  single-child inline shortcuts consume leaf children inside the parent's
  visit. It is also the first strangler target (P2-11), so it sets the
  precedent at the smallest delta.
- **`stress-wide` is a floor, not a win.** One element with 100 attributes is
  3–4 visits on every backend: attribute width costs prop work, not traversal
  work. It stays in the ladder as the control row — a phase-2 change that
  moves it is measuring something other than traversal.

## What the probe counts

A **visit** is counted where a stage's descent dispatches on a template node's
kind to decide how to continue; a **walk** at a stage's root entry only. The
19 instrumented sites and, more importantly, the **two excluded classes** are
enumerated in the module documentation
(`crates/vize_atelier_core/src/walk_probe.rs`) and repeated here because the
exclusions are what make this an approximation:

- **Subtree queries are not counted** — `collect_helpers`, `slots::detect`,
  the namespace check, `static_vnode`, `hoist_static` and its `static_type`
  classifier, and Vapor's `count_dynamic_element_children`,
  `is_static_element` and `generate_element_template`. They walk a subtree to
  answer a question or build a static string rather than to run a stage over
  it.
- **Emission shortcuts are not counted** — the single-child inline in
  `codegen::children`, the single-child unwrap in `codegen::v_if::branch`, and
  the text concatenation in `codegen::slots::generate`. They consume a leaf
  child inside the parent's visit instead of descending into it.

Both classes are real traversal work, so the totals above **understate** the
pipeline's traversal cost. They are excluded deliberately: the number this
table exists to be compared against is the one P2-12b's budget observer will
report, which counts passes visiting nodes. Counting queries here and not
there would make the comparison invalid in the direction that flatters phase 2
— the failure mode the assurance doctrine exists to prevent. The understatement
is therefore recorded as a **named limitation of the baseline**, not as a
pending fix, and the exclusion list is what a re-cut of this decision would
have to argue against.

`vize_atelier_vapor::generate` walks Vapor IR rather than the template tree,
and `vize_croquis` walks the script AST; neither is a template traversal and
neither is in scope.

## Where the numbers live

- `budgets.toml [traversal]`, keyed `<backend>_<fixture>` — 18 entries,
  gated exactly, reconciled against the ladder in both directions by
  `tests/tooling/davinci-traversal-budgets.test.ts`.
- `crates/vize_atelier_{dom,ssr,vapor}/tests/davinci_walk_baseline.rs` — the
  same numbers pinned as ordinary integration tests, so they run in the
  default `cargo test --workspace` lane rather than a feature-gated one (the
  P1-5/P1-7 counter-law shape).
