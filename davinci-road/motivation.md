# Davinci — Motivation

> [!NOTE]
> Every claim on this page was verified against the codebase on 2026-08-13.
> File paths are given so each claim stays checkable as the tree moves.

The one-line diagnosis: **Vize has a shared parse AST but no shared IR after it.**
Everything below is a consequence of that missing middle.

## Fault lines

### 1. Surface AST and codegen nodes share one type family

`vize_relief`'s `NodeType` enum spans both worlds: variants 0–12 are the parse
AST, 13–26 are JS/SSR codegen nodes (`crates/vize_relief/src/relief/codegen.rs`).
Transforms mutate the AST into a codegen tree in place — the `@vue/compiler-core`
design, inherited faithfully. There is no point in the pipeline where "what the
author wrote" and "what we will emit" exist as separate, inspectable artifacts.

### 2. Vapor escaped the AST, and pays twice for it

Vapor could not express its output on the shared tree, so it has its own IR
(`crates/vize_atelier_vapor/src/ir.rs`) — the only genuine second-stage IR in the
repo. But its entry point (`crates/vize_atelier_vapor/src/compile.rs`) first runs
the **entire VDOM-oriented transform lane and then discards the result**,
re-lowering from the AST with its own `TransformContext` that carries no scope
chain, no binding metadata, and a `Vec<String>` for diagnostics. Its directive
transforms (`v_if`, `v_for`, `v_on`, `v_bind`, `v_model`, slots, text) duplicate
`vize_atelier_core/src/steps/` nearly name for name.

### 3. Template expressions are strings

`JsExpression` is a `PhantomData` stub (`crates/vize_relief/src/relief/expressions.rs`);
the real content is `raw: String`. Every pass that needs JS structure re-parses
the string with oxc into a **fresh throwaway `oxc_allocator::Allocator`** — over
twenty call sites across `vize_croquis` and `vize_atelier_core` (e.g.
`crates/vize_croquis/src/drawer/helpers/identifiers/ast.rs`,
`crates/vize_atelier_core/src/codegen/patch_flag.rs`) — and copies results back
out as `CompactString`s because nothing can outlive the local arena. Identifier
prefixing (`_ctx.`, `$setup.`) is string rewriting
(`crates/vize_atelier_core/src/steps/expression/`), and a
hand-rolled fast/slow split makes correctness depend on a byte scanner agreeing
with a real JS parser. Root cause: the template arena (bumpalo via `vize_carton`)
and the JS arena (`oxc_allocator`) cannot share a lifetime.

### 4. Three parsers read the same `.vue` text

- `vize_armature` — the canonical parser.
- `vize_glyph` — the formatter never touches Relief; it re-scans template bytes
  itself (`crates/vize_glyph/src/template/formatter.rs`).
- `vize_musea` — the art parser is another hand-rolled scanner
  (`crates/vize_musea/src/parse.rs`).

The documented "no risk of parser disagreement" invariant currently holds only
for compile/lint/type-check.

### 5. Two virtual TypeScript generators, two source-map models

`vize_canon` (`src/virtual_ts/`, `VizeMapping`/`VizeSubSpan`) and `vize_maestro`
(`src/virtual_code/`, Volar-style `SourceMapping`/`MappingFeatures`) generate
virtual TS independently. Within canon alone, diagnostic messages are assembled
on two independent paths (persistent session vs Corsa CLI), so a fix applied to
one silently misses the other. SSR and Vapor emit **no source maps at all**, and
SFC-level maps are _recovered_ by matching emitted lines back to authored text
(`crates/vize_atelier_sfc/src/source_map.rs` is candid about this).

### 6. Node and span costs on the hot path

`SourceLocation` carries an owned `source: String` copy **per node**, plus
line/column fields the parser does not reliably fill (the source-map builder
re-derives them from byte offsets anyway). Owned strings in deep trees force
manual `Drop` impls guarded by `ensure_sufficient_stack`
(`crates/vize_relief/src/relief/{elements,control_flow}.rs`,
`crates/vize_atelier_vapor/src/ir_drop.rs`). Transform traversal walks raw `*mut`
pointers with hand-audited `unsafe` (`crates/vize_atelier_core/src/lane.rs`).

### 7. No incrementality substrate

There is no query system and no shared artifact identity. `vize_maestro` re-runs
`parse_sfc` from raw text in **63 request-path call sites**; one keystroke
regenerates every virtual document for the file. Block-level virtual-TS reuse
(#698) and Corsa session reuse (#699) are both stubs whose comments say they are
waiting on structure that does not exist. What caching does exist is a scatter of
ad-hoc result caches keyed on content hashes.

### 8. The lint rule corpus is dialect-asymmetric, and the analyzer is stranded

Patina's rule corpus is SFC-rich — 345 rule files — but JSX receives only the
subset migrated onto the markup facade, and the facade's JSX hot path
deliberately bypasses the JSX→Relief lowering (`MarkupDocument::from_jsx`)
because Relief is Vue-shaped. Sharing by _lowering into Vue's tree_ makes every
other dialect a second-class citizen; the rules need a genuinely neutral
abstraction to target.

The same strandedness holds for semantic analysis itself. Croquis computes ~25
tracker products, and most have no consumer: `RaceConditionTracker` and
`ProvideInjectTracker` are read by **nobody** outside croquis, `EffectGraph`
reaches only Doctor — never the Vapor backend, its natural consumer (Vapor
imports exactly one croquis helper, `builtins::is_global_allowed`, and the
transform lane's `Option<&Croquis>` is `None` on the Vapor path). Only 26 of
patina's 345 rule files reference croquis at all; type-aware rules bypass it
for direct Corsa sessions. Full measurement in
[Semantic Engine](./semantic-engine.md#the-problem-measured).

### 9. The rearchitected pipeline has no regression instrumentation

No crate in the template pipeline (`vize_armature`, `vize_relief`,
`vize_croquis`, `vize_atelier_core`, `_dom`, `_vapor`, `_ssr`) has a criterion
benchmark. Only end-to-end SFC benches exist. Separately,
`docs/content/architecture/performance.md` describes string interning that is not
implemented (what exists is `CompactString` SSO plus `phf` membership sets).
Davinci should make that claim true rather than delete it.

## Assets Davinci builds on

These are proofs, not aspirations — each one already works in-tree.

| Asset                       | Where                                                            | What it proves                                                                                                                                                                                                                         |
| --------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| JSX lowering                | `crates/vize_atelier_jsx`                                        | A second surface syntax can lower into one shared representation and reuse every backend, the type checker, and the linter. Davinci generalizes exactly this pattern.                                                                  |
| Markup facade IR            | `crates/vize_patina/src/{markup,ir}.rs`                          | A zero-copy, dialect-neutral _view_ over Vue templates and raw OXC JSX can drive shared rules. `ir.rs` already reserves Svelte/Astro variants. This is the precedent for HIR consumer views.                                           |
| Legacy dialect capabilities | `crates/vize_armature/src/legacy.rs`                             | A dialect can be resolved **once per file** into a capability struct, hot paths read fields only, and an off-by-default cargo feature keeps the cost of the default path at zero. This is the template for every Davinci dialect gate. |
| Real-project corpus         | `tests/_fixtures/_git` + `tools/fixtures/tool-matrix-report.mjs` | 134 pinned projects, ~35k `.vue` files, with compiler/linter/formatter/type-checker oracles already trusted (536 runs, 0 failures). This is the migration parity oracle.                                                               |
| Profiler                    | `vize_carton::profiler`                                          | Nested-span instrumentation (`profile!`) is already threaded through parse/transform/codegen; pass-level timing comes almost for free.                                                                                                 |
| Cache identity contract     | `crates/vize_doctor/src/cache_identity/`                         | Domain-separated cache keys with explicit invalidation reasons — the most principled caching design in the repo, and the seed of the stage-artifact keying scheme.                                                                     |
