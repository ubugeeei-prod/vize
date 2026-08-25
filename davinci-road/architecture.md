# Davinci — Architecture

> [!NOTE]
> This is the design-phase architecture. Stage contracts are stable; new
> implementation code uses the short stage aliases recorded in
> [charter #11](./README.md#decided-positions). The exact alias-to-package map
> and one-way dependency graph live in the
> [stage dependency policy](./plan/stage-dependencies.md). Historical art-name
> package ids stay visible during the mechanical rename window.

## What we take from MLIR, and what we refuse

**Taken as philosophy:**

- **Progressive lowering** — several IRs, each optimized for what its consumers
  ask of it, connected by explicit lowering passes. Never one tree that mutates
  itself into its own output.
- **Dialect coexistence** — do not force premature unification. A normalized core
  (`ui.if`, `ui.for`, `ui.element`) can carry framework-specific operations
  (`vue.custom_directive`, vue2 filters) alongside it, lowered later or passed
  through to a consumer that understands them.
- **Verification** — each stage has an invariant checker that runs between passes
  in debug builds and in fixtures, never in release hot paths.
- **Textual round-trip for testing** — every stage dumps to a stable, readable
  text format so fixtures and snapshots can pin any intermediate step, not just
  final output.

**Refused as machinery:** the uniform `Operation` structure, dynamic dialect
registries, and runtime-extensible type systems. In Rust those cost memory
locality, branch prediction, and type safety on every hot loop, and they fight
the workspace's clippy discipline. Each stage is a concrete typed enum.

## Stage model

```text
S0  Source model      container + spans + arena
S1  Surface trees     lossless per-dialect syntax (what the author wrote)
S2  Semantic IR       normalized, input-neutral UI semantics (what it means)
S3  Reactivity IR     static/dynamic partition, effects (how it updates)
S4  Emission          structured emitters per target (what we produce)
```

### S0 — Source model

New Davinci implementation code imports this layer as `vize_s0`; its retained
Cargo package id is `vize_carton`. The alias makes the layer position primary
without breaking published package identity.

The container layer: SFC descriptor, block boundaries, one **span coordinate
system** (`Span { start: u32, end: u32 }`, byte offsets into the authored file),
and **one arena per file compile** — under file-parallel execution (#33) each
rayon worker owns a pooled arena, so arenas are never shared across threads
or files. The arena is `oxc_allocator` (bumpalo underneath),
shared by template structures and oxc JS ASTs so both live under the same
lifetime `'a`. Strings are `&'a str` slices of the source or arena-interned
atoms — no owned strings in nodes, which also deletes the manual-`Drop`
stack-overflow class entirely. Line/column exist only at diagnostic-rendering
time, derived from offsets.

**Arena vs cache lifetime contract:** arena-backed values are per-compile
ephemera and never cross into caches. Anything cached or persisted (stage
artifacts, summaries, folio dumps, fact α-entries) is an **owned or
serialized form** — caches never hold `&'a` references and never pin an
arena. `Allocator::reset` therefore has one rule: every retained artifact is
converted to its owned form before reset, enforced by a debug arena-generation
counter and a pool-focused Miri/ASan lane.

### S1 — Surface trees (input dialects)

One lossless syntax tree per input dialect: Vue template, oxc program for
script/JSX, pug. Lossless means the formatter and lint autofixes can be written
against S1 without a private re-scan — this is what retires the `vize_glyph`
byte scanner and the `vize_musea` hand parser. Error tolerance is
**structural**, SwiftSyntax-style: malformed source becomes typed
`Unexpected` nodes and absent-but-required tokens become `Missing` tokens, so
every consumer sees one uniformly-shaped tree with holes and S1→S2 has a
single documented hole policy. The debug verifier asserts
`render(tree) == source` bytes on every construction.

For script/JSX, S1 is an **OXC-backed lossless wrapper**, not raw
`oxc_ast::Program`: OXC's default parser config discards tokens
(`ParserReturn` owns them, trivia retention is opt-in) and its error recovery
yields diagnostics plus a structurally-valid (or, on panic, empty) AST — not
typed holes. The wrapper enables token/trivia retention, owns the source
text, and maps OXC recovery outcomes into the `Unexpected`/`Missing` model,
and only that wrapper carries the lossless/round-trip guarantee the formatter
and autofixes rely on. Vue 2 is an S1/S2 dialect using
the existing `legacy` capability model (resolve once per file, feature-gated,
zero cost when off).

### S2 — semantic IR (the pivot; crate `vize_s2`, codename Disegno)

The normalized, input-neutral representation of UI semantics, and the **primary
consumer surface**: element/component/text/interpolation nodes, structured
control flow (`if`/`for` as regions, not directive attributes), normalized slots,
normalized bindings (`bind`/`on`/`model` semantics rather than `v-bind`/`v-on`
spellings), with semantic facts attached via side tables (the
[Semantic Engine](./semantic-engine.md)). JSX `<Show>`-style patterns, `v-if`,
and pug conditionals all normalize to the same ops. Framework-specific
constructs that must survive (custom directives, vue2 filters) ride along as
dialect ops.

**The neutral core is a fair abstraction, not Vue's AST renamed.** Vue lowers
into it exactly the way JSX or an external dialect does; whatever is genuinely
Vue-specific stays a `vue.*` dialect op instead of shaping the core. The litmus
test: a lint rule written against the neutral core runs unchanged on SFC and on
JSX — and on Svelte/Solid through the input-dialect contract — wherever the
underlying semantics exist. Today fails that test: Patina's SFC rule corpus is
rich (345 rule files) while JSX gets a migrated subset, and the JSX hot path
deliberately bypasses the JSX→Relief lowering (`MarkupDocument::from_jsx`)
because Relief is Vue-shaped. Lowering _into a Vue-shaped tree_ is the wrong
fix; a genuinely neutral S2 is the right one.

**Two-way binding — contract vs realization.** `v-model` is the instructive
boundary case: it is _not_ sugar for `:value` + `@input` — the runtime
realization guards IME composition events, handles checkbox arrays, `.lazy`'s
change-vs-input switch, and select-multiple. So the neutral core carries
`ui.model` as the **binding contract only** (what is read, what is written,
the value-type flow — which is all lint, the lattice, and type projection
need, and which Svelte's `bind:` lowers to identically), with element kind
and dialect modifiers riding as attributes. **Realization is never expanded
in S2**: each S4 target picks it at lowering — VDOM emits runtime directive
references (as Vue does today), Vapor calls upstream vapor helpers, SSR
renders attributes. IME/composition handling is **runtime-owned by
declaration**; the compiler's obligation is to select the correct realization
and preserve the contract, never to reimplement composition. Composition
behavior is pinned by behavioral-tier tests with IME event scripts.

S2 also crosses SFC block boundaries where semantics do: `v-bind()` in CSS
appears as S2 binding ops, so the linter, the reactivity lattice, and the
type-check projection see style-block references instead of leaving them a
descriptor-level blind spot.

Consumers: the linter (Patina's markup facade becomes a zero-copy view over S2,
and the rule engine targets the neutral core), virtual-language projection for
type checking, Musea, Doctor, and LSP features.

### Expression dialects

Because expression languages are themselves pluggable (decision 4 — MoonBit,
Elixir-hosted expressions), S2 does not hard-wire expressions to oxc:

```rust
enum ExprRef<'a> {
    /// Fast path: JS/TS parsed by oxc into the shared arena. In-tree default.
    Js(&'a oxc_ast::ast::Expression<'a>),
    /// Foreign expression dialects, feature-gated like `legacy`.
    Foreign(&'a ForeignExpr<'a>), // dialect id + source slice + span + side tables
}
```

Every expression dialect implements one capability contract, resolved per file,
never dyn-dispatched per node: enumerate referenced bindings (drives scope
analysis, patch flags, effect dependencies), classify static/const-ness, map
spans, and emit for a given target. For JS these are direct oxc AST walks — the
fast/slow byte-scanner split disappears because the parsed AST is simply kept.

Type checking generalizes the same way: canon's virtual TS becomes the JS
instance of a general **virtual host-language projection** — an S4 target that
emits checkable code plus span links for any expression dialect (virtual MoonBit
for MoonBit expressions, delegated to the host toolchain the way TS is delegated
to Corsa today). **Decided:** this projection duty is part of the
expression-dialect contract itself, not an optional extra — a dialect that
cannot emit a checkable projection with span links only qualifies for
boundary-typed (opaque) integration.

The projection's span-link data is designed for three consumers at once: the
Corsa/tsgo API surface (native project sessions), the existing
**content-mapper protocol** (`vize content-mapper`, the tsserver-plugin-style
host interface), and Maestro's editor features. One mapping model, three
transports — this is what retires the current canon/maestro mapping split.

### S3 — reactivity IR (future alias `vize_s3`, planned package `vize_impeto`)

Named for Leonardo's concept of impetus — how motion propagates. The
generalization of today's Vapor IR: flat, id-based operations
(`SetText`/`SetProp`/`InsertNode`/…), static template partition, effect grouping
by dependency set, and hoist/cache decisions as explicit operations rather than
codegen-time inference. The partition derives from the semantic engine's
[reactivity lattice](./semantic-engine.md#the-reactivity-lattice--one-analysis-every-backend),
computed once and serving all three backends. **Decided routing:** DOM and
Vapor lower through S3 — patch flags and effect grouping are both "reactivity
decisions" and belong in one place — while SSR, which has no update phase,
takes a thin S2→S4 path and reads the static partition as semantic-engine
facts. Phase 3 measurements retain veto power over this split.

Three design commitments from the literature (see [Prior Art](./prior-art.md)):
ordering constraints are **explicit state edges** (RVSDG-style), not implicit
walk order, so partition and grouping decisions are local graph queries;
placement alternatives (hoisted / cached / inline / grouped-effect) stay
explicit on the node and are resolved at **one cost-driven extraction point**
— executed Flambda2-style as _try-measure-commit_: perform the candidate,
simplify locally with fact-engine approximations in scope, measure (emitted
size, reactive-edge count, update-path length), commit only on measured
benefit under a decrementing per-component budget; and correctness has a
mechanical oracle from the IVM framing — _incremental update output ≡
from-scratch render_ — with patch flags and SSR plans derived from operator
linearity (a keyed `v-for` is a linear operator; non-linear mixes are where
cache ops belong).

### S4 — Emission (output targets)

A structured emitter layer replaces string-append codegen: targets build a span-
carrying document, and source maps fall out of emission for **every** target —
DOM, SSR, and Vapor alike — replacing the text-matching recovery in
`vize_atelier_sfc/src/source_map.rs`. Targets are: VDOM JS, Vapor JS, SSR JS,
virtual TS / virtual host-language projections, `.d.ts`, and non-JS host targets
(the Volt/Elixir pattern) through the same contract.

## Stages are contracts, passes are execution plans

The stage model is **logical**. S0–S4 define data contracts, dump formats, and
consumer surfaces; they do not mandate five traversals. Passes declare
themselves **fusable** (single-visit, local, synthesized-attribute style) or
**barrier** (needs whole-tree or fixpoint facts), and the pass manager fuses
adjacent fusable passes into one walk. Physical plans then differ per product:

- **`vize build` fuses aggressively.** Parsing can emit S2 directly — S1 is a
  _capability_, materialized only when a consumer needs losslessness (the
  formatter, lint autofix). Cheap semantic facts are computed as synthesized
  attributes during lowering; emission runs as the exit action of the final
  walk where the target allows. The budget is explicit: the fused compile path
  must not walk the tree more times than today's pipeline — which is already
  parse + transform + hoist + codegen plus 20+ per-expression re-parses, and
  for Vapor an additional discarded transform and re-lower. Multi-stage IR done
  right _reduces_ traversals here; it does not add them.
- **`vize check`, lint, and the LSP materialize.** They query S2 and fact
  tables repeatedly and incrementally, so artifact caching (phase 5) dominates,
  not traversal count.

Region-structured control flow in S2 is what makes fusion tractable: today's
enter/exit sibling-mutation dance (merging `v-else` branches on the parent's
child list) forces the re-visits that a region-owning `ui.if` op never needs.

## Shared infrastructure and extension contracts

Moved to [Architecture — shared infrastructure and extension contracts](./architecture-infrastructure.md) under the 350-line source budget: the pass manager, Folio, provenance, the diagnostics channel, verifiers, and the decision-1 extension contracts.

## Priority order (charter #22)

When designs conflict: **performance and correctness win, always** — over
size, over extensibility, and especially over implementation simplicity,
which is explicitly not a goal. "Performance" means throughput, latency,
**and memory** — peak and steady-state — as one budget family. Concretely licensed by this ordering:
`unsafe` with verifier-checked invariants, hand-specialized data structures
and algorithms over library convenience, elaborate fusion machinery, and
per-target monomorphized codepaths. The license has one boundary: complexity
must live inside the disciplines (typed contracts, stage verifiers, Folio
inspectability) — hard code with a verifier and a dump is engineering; hard
code without them is a hack. Size defends itself only through its budgets
(#19); extensibility only through the two-tier contract model (#15).

## Performance guardrails

Non-negotiable, inherited from "Be Fast Above All":

1. **No dyn dispatch in per-node hot loops.** Dialect and pass dispatch happen
   per file or per pipeline, never per node.
2. **One arena, zero re-parses.** An expression is parsed exactly once per
   compile; keeping the AST must be cheaper than today's parse-copy-reparse.
3. **Spans are two u32s.** No owned strings, no eagerly-computed line/column.
   More generally, **node sizes are pinned**: every stage node type carries a
   `static_assert`-style size test (the rustc practice), so a refactor cannot
   silently fatten a hot node; per-stage bytes-per-node accounting is part of
   the microbench suite, and allocation counts (the profiler's existing
   allocation tracking, promoted to CI metrics) are budgeted alongside time.
   Batch compilation is block-streaming — whole-project peak memory stays
   proportional to the largest block in flight, never to project size — and
   arenas are reused across files (`Allocator::reset`), not reallocated.
4. **Every phase holds the budget.** The end-to-end benchmark envelope
   (15k SFC ≈ 330ms compile today) is a merge gate whose **normative
   definition lives in [`plan/budgets.toml`](./plan/budgets.toml)**
   — seeded now with the envelope's machine, statistic, run count, cache
   state, parallelism, baseline, and tolerance (from the committed Blacksmith
   snapshot); per-crate budgets land via plan task P0-4 — and phase 0 adds
   the per-crate microbenches the pipeline currently lacks so regressions
   localize.
5. **Verification never ships.** Stage verifiers are debug/fixture-only.
6. **Traversal count is budgeted.** The fused compile path must not exceed the
   current pipeline's number of tree walks; phase-0 microbenches make fusion
   regressions localizable.
7. **Resource budgets, not just throughput budgets.** The anti-goals are named:
   the "rust-analyzer is too heavy" and "cargo build is too slow" failure
   modes. Resident processes carry CI-tracked ceilings for RSS, cold-start
   time, keystroke latency, and idle CPU (an idle server burns ~zero); one-shot
   commands carry cold-run wall/RSS budgets. Fast, stable, and economical are
   one requirement, not three.
8. **Distribution size is budgeted too.** Native binary, wasm blob, and npm
   package sizes are CI-tracked with ceilings. Feature gating (the `legacy`
   pattern) and the two-tier contract model (external dialects never compiled
   in) are what keep the default artifact lean.

## Portability: `no_std` core, WASI as a first-class target

Davinci-owned crates (`vize_davinci`, `vize_s2`, `vize_s3`/`vize_impeto`) are
written `no_std + alloc` from birth: stage data, passes, and emitters depend on
the arena and core types only, with `std` gated to the edges (filesystem,
threads/rayon, process spawning, clocks). CI builds the core for
`wasm32-wasip2` alongside native targets. This is what "runs everywhere" means
concretely: browsers and the playground via wasm, edge runtimes, and embedding
inside non-JS hosts (an Elixir NIF, a MoonBit host) without dragging a
platform layer along. Existing dependencies (oxc, lightningcss) set the
practical boundary — where they require `std`, the seam is documented rather
than fought (see [Open Questions](./open-questions.md)).

## Observability: Folio, the DevTool, and the AI optimization loop

Three layers share one data model:

1. **Folio dumps** carry _what_ each stage holds; every op records provenance
   (which pass/rule produced it — by name, so WASM extensions get first-class
   provenance — from which source node, with before/after pairs at lowering
   decisions). Provenance **survives failure**: partial S2/S3 fragments are
   kept on error, Lean-InfoTree style, so the LSP and DevTool stay live on
   broken SFCs. In the fused CLI walk provenance is off or ring-buffered;
   resident/DevTool mode materializes it fully.
2. **Source-level profiling** — `vize_carton::profiler` is extended so `profile!`
   spans attribute cost to pass × stage × file/block × source span, exported in
   a stable machine-readable schema (the `vize_doctor::ai_context` precedent:
   budgeted, vendor-neutral payloads).
3. **The [DevTool](./devtool.md)** renders both live: stage-by-stage lowering,
   pass-by-pass diffs, fact tables, the reactivity lattice, and per-pass flame
   views. Reconciliation with the CLI's ring-buffered provenance: Spolvero's
   live views attach to **resident or replay executions** (where provenance
   is fully materialized); inspecting a one-shot CLI run means replaying it
   (`vize repro` / `davinci-opt`) rather than expecting the fused walk to
   have retained everything.

The same artifacts close the **AI optimization loop**: profiles and Folio
diffs are structured input an agent can consume, and the corpus + benchmark +
budget gates are the oracle that verifies any AI-proposed optimization. Human
or AI, the gate is the same — optimization becomes a loop that can run
unattended without lowering the bar.

## Fit with workspace culture

New crates start at the `experimental` stability tier and obey the existing
discipline: `vize_s0` string/collection types (from the `vize_carton` package;
clippy bans), the 350-line
source guard (**explicitly reaffirmed for Davinci crates** — charter #22's
complexity license and file splitting are orthogonal; small files serve
reviewability, which serves correctness), fixtures-first change classes from
`docs/content/architecture/language-engineering-practices.md`, and snapshot
diffs as reviewed contracts — which the Folio dumps are designed to serve.
