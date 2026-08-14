# Phase 1 — One Arena, Real Expressions

> [!NOTE]
> The highest-leverage phase: expressions parsed once, strings out of nodes,
> one lifetime. Exit requires a measurable performance **win**, not parity.
> Dependency chain is real here — order matters. In-phase fallback flags per
> charter #26; all deleted at P1-13.

## TODO index

- [x] P1-1 Arena unification in carton
- [ ] P1-2 Allocator plumbing through armature/atelier
- [ ] P1-3 `SourceLocation` diet: retire per-node `source` strings
- [ ] P1-4 `SourceLocation` diet: retire dead line/column
- [ ] P1-5 Retained expressions: parse-once storage
- [ ] P1-6 Consumer migration wave A (croquis identifier/scope helpers)
- [ ] P1-7 Consumer migration wave B (atelier patch-flag, v-for, transforms)
- [ ] P1-8 Delete the fast/slow scanner split
- [ ] P1-9 Identifier prefixing as AST transform
- [ ] P1-10 Node strings → `&'a str` / arena atoms; delete manual `Drop`s
- [ ] P1-11 Arena reuse across files (batch pool)
- [ ] P1-12 Performance-doc truth pass
- [ ] P1-13 Phase exit: budgets pinned, old paths deleted

---

## P1-1 — Arena unification in carton

**Steps:**

- [x] `crates/vize_carton/src/allocator.rs`: `Allocator` becomes the unified per-compile handle `{ bump: Bump, oxc: oxc_allocator::Allocator }` — one value, one lifetime, one `reset()` covering both pools; `as_oxc()` exposes the retained-AST pool P1-5 parses into _(amended in-PR: the original step assumed oxc_allocator was still bumpalo-backed and the container aliases could flip here. At the pinned rev (0.142, `fc702c1`) oxc ships its own `Arena` — no bumpalo inside — and `oxc_allocator::{Box, Vec}` reject `Drop` payloads with a const assertion, which today's string-owning nodes trip. The aliases flip at P1-10 once the string diet makes nodes `Drop`-free; until then the two-pool handle is the chartered in-phase transitional state (#26))_
- [x] Audit bumpalo-API deltas used in-tree _(finding: `Deref<Target = Bump>` is preserved, so every `Box::new_in`/`Vec::new_in`/`BumpString`/`BumpVec` call site compiles unchanged; `alloc_str`/`as_bump`/`reset` keep their signatures; `allocated_bytes` now reports both pools' bytes — sole external consumer is vize_maestro's virtual-code metrics, where the sum is the truthful number; nothing needed a shim)_
- [x] Node-size static asserts from P0-9 unchanged _(no node type changes in this task)_

**Acceptance:** workspace compiles, full test suite green, P0 benches within noise, `tools/davinci/corpus-diff.mjs` empty.
**Deps:** P0-4, P0-9.

## P1-2 — Allocator plumbing

**Steps:**

- [ ] Thread `&'a Allocator` through `vize_armature` entry points (`crates/vize_armature/src/parser/entry.rs`, `parser.rs`) and the atelier lanes (`vize_atelier_core::lane::transform`, sfc `compile_template`) so template structures and future oxc ASTs share `'a`
- [ ] `vize_atelier_jsx` (already oxc-based) switches its local allocator to the caller-provided one

**Acceptance:** corpus-diff empty; benches hold; no public-API regressions the CLI/vitrine can't absorb (charter #23: internal breakage is free).
**Deps:** P1-1.

## P1-3 — Retire per-node `source` strings

**Steps:**

- [ ] `SourceLocation` (in `crates/vize_relief/src/relief/core.rs`): remove `source: String`; add `span: Span` (P0-9 type); every consumer from `davinci-road/plan/sourcelocation-inventory.md` switches to `Span::slice(source_text)` — inventory rows checked off in the PR
- [ ] Diagnostic excerpt rendering reads via span + the file's source text (threaded where missing)
- [ ] Re-pin node-size static asserts (they shrink)

**Acceptance:** corpus byte parity **including diagnostic text**; alloc counts drop in P0-2 benches (record the delta); inventory file updated to all-migrated.
**Deps:** P1-2, P0-9.

## P1-4 — Retire dead line/column

**Steps:**

- [ ] `Position` reduced to `offset: u32` (or `Position` deleted in favor of `Span`); line/col derived only in diagnostic rendering and `SourceMapBuilder::finish()` (`crates/vize_atelier_core/src/codegen/source_map.rs` — it already re-derives from offsets because parser line info was unreliable)
- [ ] Delete the parser's vestigial line-tracking fields

**Acceptance:** corpus parity including source maps; node sizes re-pinned smaller.
**Deps:** P1-3.

## P1-5 — Retained expressions: parse-once storage

**Steps:**

- [ ] `crates/vize_relief/src/relief/expressions.rs`: `JsExpression<'a>` replaces the `raw: String + PhantomData` stub with `ast: &'a oxc_ast::ast::Expression<'a>` (+ `raw: &'a str` slice for display); decide parse point by bench — during template parse (armature) vs first semantic touch (croquis) — record the measurement in the PR
- [ ] Parse via `oxc_parser` with the shared arena from P1-2; template-expression parse errors keep today's diagnostic behavior (differential-checked)
- [ ] Profiler counter `davinci.expr.parses` incremented at the single parse site; exported via P0-11
- [ ] Lifetime note enforced: retained `&'a Expression` values are per-compile ephemera under the architecture's arena/cache contract — anything crossing a compile boundary (caches, folios, summaries) uses the owned serialized form, never the arena reference

**Acceptance:** the counter law matches the chosen policy — **each expression
parsed at most once (zero re-parses) always**; under parse-at-template-parse,
counter == distinct expressions; under parse-on-first-touch, counter ≤
distinct expressions and untouched expressions are provably never parsed.
Corpus parity; benches hold or improve.
**Deps:** P1-2.

## P1-6 — Consumer migration wave A (croquis)

**Steps:**

- [ ] `crates/vize_croquis/src/drawer/helpers/identifiers/slow.rs` — reads the retained AST instead of re-parsing (its local `Allocator::default()` dies)
- [ ] `crates/vize_croquis/src/drawer/helpers/v_for/oxc.rs`, `visit_element/second_pass.rs` — same
- [ ] Differential lane: for one release, CI compares old-path vs new-path results (identifier sets, v-for destructure shapes) over the corpus; divergences are bugs in one side — investigate, don't average
- [ ] Charter #37 note: only the _input layer_ moves; tracker internals untouched

**Acceptance:** differential lane green over the corpus; `davinci.expr.parses` drops (record); croquis bench improves.
**Deps:** P1-5.

## P1-7 — Consumer migration wave B (atelier)

**Steps:**

- [ ] `crates/vize_atelier_core/src/codegen/patch_flag.rs` — retained AST, local arena dies
- [ ] Remaining reparse sites from the P0-3 counter baseline (~20 in `vize_atelier_core`), checklist generated from that baseline and checked off per site
- [ ] Same differential-lane pattern as P1-6

**Acceptance:** `davinci.expr.parses` reaches its floor (== distinct expressions) on the corpus — asserted in CI from the profiler export; parity holds.
**Deps:** P1-5 (parallel with P1-6).

## P1-8 — Delete the fast/slow scanner split

**Steps:**

- [ ] Pre-deletion differential run: byte-scanner (`identifiers/fast.rs`) vs retained-AST walk over the whole corpus — committed report proving agreement (or documenting scanner bugs the AST walk fixes; those are waiver-reviewed)
- [ ] Delete `identifiers/{fast,slow}.rs` and the dispatch; single AST-walk implementation remains
- [ ] Bench check: if the scanner was faster on `stress-*` fixtures, the walk must close the gap before deletion merges (measured, not assumed)

**Acceptance:** grep zero for the deleted modules; corpus parity; croquis benches hold or improve.
**Deps:** P1-6, P1-7.

## P1-9 — Identifier prefixing as AST transform

**Steps:**

- [ ] Replace string rewriting in `crates/vize_atelier_core/src/transforms/transform_expression/{prefix,rewrite,nesting}.rs` with an AST-level transform over retained expressions (scope-aware via croquis bindings) and span-preserving emission
- [ ] Source-map assertions for rewritten identifiers added to the P0 fixture set
- [ ] **Waiver budget: zero.** Prefixing output is the most visible codegen surface; any corpus diff is a bug in this task

**Acceptance:** corpus byte parity on dom/vapor/ssr; new source-map assertions green.
**Deps:** P1-7.

## P1-10 — Node strings to `&'a str` / atoms; delete manual `Drop`s

**Steps:**

- [ ] Interner in `vize_carton` (arena-backed atoms for names appearing repeatedly: tags, directive names, well-known attrs); node fields (`name`, `tag`, `content`) become `&'a str` slices or atoms — per-field decision recorded in the PR
- [ ] Collapse the P1-1 two-pool handle: flip `vize_carton::{Box, Vec}` aliases to the oxc arena types (their no-`Drop` const assert passes once nodes are string-free), delete the `bump` pool from `Allocator`, and retire the `Bump`/`BumpString`/`BumpVec` re-exports
- [ ] Delete `ensure_sufficient_stack` `Drop` impls: `crates/vize_relief/src/relief/elements.rs`, `relief/control_flow.rs`, `crates/vize_atelier_vapor/src/ir_drop.rs` (arena drop is free once nothing owns heap strings)
- [ ] `stress-deep.vue` fixture passes without the stack guard (the guard's reason is gone, prove it)
- [ ] The `docs/content/architecture/performance.md` interning claim becomes true — note for P1-12

**Acceptance:** grep zero for manual `Drop` on node types; alloc counts drop (pin the number as the new ratchet); corpus parity.
**Deps:** P1-3.

## P1-11 — Arena reuse across files

**Steps:**

- [ ] Pool in the CLI batch path (`crates/vize/src/commands/build/`): per-rayon-worker `Allocator` reset between files (`Allocator::reset()`, available in the pinned oxc_allocator), not reallocated
- [ ] **Lifetime contract documented and enforced:** every arena-backed value is consumed or converted to its owned form (stage artifacts, cached results, diagnostics) _before_ reset — caches never hold `&'a` references and never pin an arena (see the architecture arena/cache contract)
- [ ] Escape check: asan/miri lane over the pool (nothing borrows across `reset`), plus a `#[cfg(debug_assertions)]` arena-generation counter that panics on cross-file survivals; includes a **resident-cache reset scenario** (cache populated → arena reset → cache read) proving cached data is owned

**Acceptance:** peak RSS on the corpus batch drops (pin as ratchet); asan/miri lane green.
**Deps:** P1-10.

## P1-12 — Performance-doc truth pass

**Steps:**

- [ ] `docs/content/architecture/performance.md`: every claim (interning, arenas, allocation behavior, string handling) rewritten to match shipped code, with numbers from this phase's benches; the stale pre-Davinci claims deleted

**Acceptance:** review point — maintainer signs off claims against `bench/results/davinci/`.
**Deps:** P1-10, P1-11.

## P1-13 — Phase exit

- [ ] Corpus compile parity: byte-identical, waiver ledger empty
- [ ] `davinci.expr.parses` satisfies the P1-5 counter law (zero re-parses; == or ≤ distinct expressions per the chosen policy) asserted in CI
- [ ] Compile bench improvement ≥ target pinned at phase start (set from P0-3's double-transform + reparse baselines; record the target in `budgets.toml` before P1-5 merges)
- [ ] Alloc count / peak RSS improvements pinned as new ratchet baselines
- [ ] Scanner split, string-rewrite prefixing, manual `Drop`s: deleted (grep zero)
- [ ] In-phase fallback flags removed (charter #26)
