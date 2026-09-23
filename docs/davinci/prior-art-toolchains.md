# Davinci — Prior Art & Imported Practices: toolchains, assurance, literature

> [!NOTE]
> Second half of the [prior-art survey](./prior-art.md) (split under the
> 350-line source budget). Same survey date (2026-08-13), same format: what
> the system does, what Davinci imports, and what it deliberately does not.

---

## Swift

**SwiftSyntax.** Lossless trees with byte-fidelity (trivia on tokens) that
render back exactly — _including malformed input_: broken source becomes
first-class `unexpected` nodes and required-but-absent tokens become `missing`
tokens, so every consumer sees one uniformly-shaped tree with holes
([SwiftParser](https://github.com/swiftlang/swift-syntax/blob/main/Sources/SwiftParser/SwiftParser.docc/SwiftParser.md)).
_Import:_ S1 encodes `Unexpected`/`Missing` as typed node kinds — no per-
consumer error special-casing, one documented hole policy for S1→S2; and the
cheapest high-yield verifier in the survey: debug-assert
`render(tree) == source` bytes on every S1 construction.
_Anti-lesson:_ don't copy the persistent value-semantic tree implementation
(arena + single writer is right for us), and don't build incremental reparse
until LSP profiling demands it — Swift's exists and is barely used.

**Macros as out-of-process plugins.** Sandboxed separate processes exchanging
serialized trees, with a `getCapability` handshake (integer protocol version +
feature strings). The famous build-time pain (7.7× clean builds) was **not**
the process boundary — it was every macro package recompiling swift-syntax
from source; the fix (prebuilt versioned binaries) took two years
([roundup](https://mjtsai.com/blog/2024/02/27/slow-swift-macro-compilation/)).
_Import:_ the capability handshake goes into the WIT world verbatim;
sandbox + determinism is the stated contract that makes extension output
cacheable; and the decisive one — **the extension SDK ships as a prebuilt,
versioned artifact from day one** (the component model gives us the stable
binary contract Swift lacked). Compatibility _policy_ is written down, not just a version number.

**Request evaluator / incremental builds.** Hashable queries with automatic
cycle detection — except cycle diagnostics are disabled in production ("too
frequent in real-world code") and the mutable-AST migration is unfinished a
decade in. `.swiftdeps` tracks fine-grained provides/depends at top-level /
nominal / member granularity.
_Import:_ per-SFC summaries split into fingerprinted **facets** (signature,
prop/emit/slot types, member-level info) so consumers invalidate only on
facets they used; and fact-group cycles are made **impossible by
stratification** (a group may demand only earlier strata), never detected at runtime.
_Anti-lesson:_ queries retrofitted onto mutable state is the negative proof
for our immutable-stage + resident-only-salsa split; and two parallel
dependency systems is accidental complexity — the fact engine is the single
source of dependency truth for both LSP and batch.

**SIL — mandatory vs optional passes.** SILGen emits _raw_ SIL; a fixed set of
**mandatory passes** (definite initialization, coverage diagnostics) runs at
every optimization level and produces _canonical_ SIL — the only form later
stages accept; optional optimizations run after
([SIL.md](https://github.com/swiftlang/swift/blob/main/docs/SIL/SIL.md)).
_Import — the most directly adoptable idea in the survey:_ every Davinci pass
is classified **mandatory-diagnostic / mandatory-lowering / optional-
optimization**; raw→canonical is a type-level transition per stage; mandatory
passes are unfusable barriers that run at every opt level and define where
diagnostics attach (dataflow-hungry lint rules become mandatory-diagnostic
passes over canonical S2/S3 — structurally ending the two-path diagnostic
assembly problem). Only optional passes participate in fusion and the
traversal budget.
_Anti-lesson:_ OSSA took ~8 years to retrofit — S3's state edges are
verifier-first from birth.

## GHC

**Core + Lint.** All of Haskell desugars to a tiny explicitly-typed core;
because binders carry types, re-typechecking is linear and local, and
`-dcore-lint` runs it between every pass ("checks GHC's sanity, not yours"),
with sibling lints at every later IR.
_Import:_ the design property, not the tool — S2/S3 nodes carry enough
redundant typing/shape information that verification is **local** (a node +
its operands' declared facts), no global inference; verifiers run between
passes (after each fused group minimum) so failures name the offending pass.
_Anti-lesson:_ type-well-formedness won't capture reactivity semantics the way
System FC's types capture Haskell's — S3's verifier needs explicit semantic
invariants (state-edge liveness, extraction preserves the edge set) beyond
"it type-checks"; budget for it.

**Interface files.** `.hi` files fingerprint the interface **per declaration**;
each module records the fingerprints of everything it _used_; recompilation is
skipped iff every used fingerprint is unchanged — the 15-years-proven rule
([Tweag deep-dive](https://www.tweag.io/blog/2022-11-03-blog_recompilation/)).
Orphan instances punch through the firewall and force reading every orphan
interface; unfoldings-in-interfaces buy optimization at the cost of cascading invalidation.
_Import:_ per-declaration fingerprints + record-what-you-used becomes the
per-SFC summary invalidation rule; the **orphan equivalent is named now** —
app-global facts (global components, app-level provide/inject, dialect-wide
directives) get a dedicated global summary with its own fingerprint instead of
pretending to be per-file; and a hard rule: **S3 code-shape decisions never
enter the interface summary** — a summary describes the contract, never the
chosen optimization, or hot-path tuning ripples recompilation project-wide.
IDE artifacts (HIE-like: positions, types-at-locations) are a separate
artifact class that never triggers recompilation — Folio's role.

**RULES & the simplifier (anti-lesson).** Library-installable rewrites with no
semantic checking, no confluence guarantees, and phase-number choreography
that remains folklore decades later; the simplifier's iterate-until-quiescent
fixpoint is why GHC compile times are unpredictable.
_Import:_ canonicalization rewrites are closed-world (third parties propose
facts, never install semantic rewrites), mechanically checked against stage
verifiers, and ordered by explicit named dependencies — never global phase
numbers. The fixed traversal budget replaces fixpoint iteration: make each
traversal do more, not run more traversals.

## OCaml Flambda2

One downward + one upward purely-functional traversal; the downward pass
carries **value approximations** (an abstract domain) and analyzes while
transforming; **speculative inlining actually performs the inline, simplifies
in context, measures retired-instructions vs size growth, and keeps it only on
measured benefit**, under a decrementing budget
([Flambda2 snippets](https://ocamlpro.com/blog/2024_08_09_the_flambda2_snippets_3/)).
`-O2`/`-O3` are the same passes with bigger budgets. The flambda/non-flambda
compiler fork is link-incompatible — a whole ecosystem manages the split.
_Import:_

- **Try-measure-commit for S3 extraction**: candidate placements (hoist /
  cache / inline / group) are performed, locally simplified with fact-engine
  approximations in scope, measured (emitted size, reactive-edge count,
  update-path length), and committed only on positive benefit under a
  per-component budget. This is the production-verified form of the deferred
  cost-driven extraction the aegraph discipline pointed at.
- **Optimization tiers as budget constants** — identical pass set, scaled
  budgets; never forked pipelines.
- **Two traversals is enough** — the strongest precedent for the traversal
  budget: a production optimizer lives on one-down-one-up if approximations
  flow with the walk; the demand-declared fact groups are our downward
  environment.
  _Anti-lesson:_ the hard fork (link-incompatible variants) is what budget-scaled
  tiers exist to avoid — all tiers must emit interface-summary-compatible
  output so mixing opt levels across SFCs never breaks a build. And aggressive
  speculation demands equivalence checking: the
  [Flambda2 Validator](https://icfp24.sigplan.org/details/ocaml-2024-papers/6/Flambda2-Validator)
  (translation validation) is the mature companion to per-stage verifiers —
  aligning with the metamorphic/differential folio oracles already planned.

## Lean 4

**InfoTree — provenance that includes failures.** Lean's elaborator builds an
[InfoTree](https://leanprover-community.github.io/mathlib4_docs/Lean/Elab/InfoTree/Types.html)
as a side product: every node records which elaborator produced it, from which
syntax, with before/after pairs for macro expansion, replayable context
snapshots — **and partial results are kept when elaboration fails**, so hover
works in broken code. All LSP features walk this one tree.
_Import:_ Davinci provenance records carry `(rule name, input node,
before/after, context)` at every lowering decision; partial S2/S3 fragments
survive errors so the LSP and DevTool stay live on broken SFCs; one structure
feeds the DevTool ladder, remarks, and hover.
_Anti-lesson:_ InfoTree is memory-heavy — provenance is ring-buffered/off in
the fused CLI walk, fully materialized only in resident/DevTool mode.

**Snapshot-tree incrementality — sub-file granularity without queries.** Lean
achieved intra-file incrementality ([4.8.0](https://lean-lang.org/blog/2024-6-1-lean-480))
and parallel elaboration ([4.19.0](https://lean-lang.org/doc/reference/latest/releases/v4.19.0/))
with no query engine: a tree of snapshot tasks at structural joints, one reuse
rule (old syntax ≡ new syntax ⇒ adopt old subtree), cascade-cancellation
tokens, and a watchdog/per-file-worker server.
_Import:_ a **third incrementality mode layered under salsa** — salsa at
file/summary granularity, snapshot-adoption inside an SFC at natural joints
(header → block → S2 region), covering most keystroke traffic without pushing
salsa finer. Cancellation tokens through every stage task; worker isolation by
threads + catch-unwind (not per-file processes — Lean's file sizes afford
those, ours don't); "header changed ⇒ restart" maps to the per-SFC summary firewall.

**Kernel discipline — verify the artifact, not the pass.** Lean's tiny kernel
re-checks elaborator output through a closed term language; its independence
is proven by external checkers ([Lean4Lean](https://arxiv.org/abs/2403.14064)).
_Import:_ stage verifiers take only the serialized stage artifact (the Folio
form) + the previous artifact, never pass internals — which makes
out-of-process verification of folio dumps in CI possible. If a verifier
imports pass code, that's the smell. Verification stays debug/CI-only (Lean
re-checks always because correctness is its product; ours is speed).

**Environment extensions — the α/β fact split.** `@[simp]`-style registries
persist a serialized entry form (α) and rebuild in-memory indexes (β) on
demand ([Lean.Environment](https://leanprover-community.github.io/mathlib4_docs/Lean/Environment.html)).
_Import:_ each fact group defines (α serialized summary entries, β in-memory
index) with an explicit export function — the α form _is_ the per-SFC summary
contract, versioned independently of the index; producers (first-party or
WASM) declare which groups they contribute to, so demand resolution and the
interface hash both derive from declarations.
_Anti-lesson:_ Lean's global environment means any import's simp lemma affects
your file (mathlib's simp-pollution fights) — fact groups stay per-SFC with
explicit summary edges, scoped by default.

**Module system & cache keys.** Lean's 2025 module system retrofits
private-by-default and body-elision so recompilation keys on interfaces
([4.22.0](https://lean-lang.org/doc/reference/latest/releases/v4.22.0/));
Lake's content-addressed cache had a documented OS-dependent-hash incident.
_Import:_ summaries are body-elided **by type construction** (day-one
invariant vs Lean's multi-year retrofit); artifact keys scoped by (toolchain,
platform, features); a CI oracle computes keys on two platforms and diffs.

**Round-trip printing.** Lean's `notation` generates parser _and_ printer from
one declaration; divergence pain lives exactly where print/parse are
hand-paired. `pp.all` mode is injective and re-elaborates; the printer
self-refines options until round-trip succeeds.
_Import:_ one declarative grammar per Folio stage generating both parser and
printer (`#[derive(Folio)]`); two modes — human-readable (may elide) and
`--full` (guaranteed injective, the round-trip oracle applies here); on test
mismatch, auto-re-print at maximal explicitness and diff. Folio stays closed —
no notation extensibility, rigid MLIR-style syntax keeps the oracle trivial.

**Metaprogramming anti-lesson.** Lean's arbitrary in-process syntax
extensibility forces environment-dependent parsing, dynamic dispatch, and
process isolation for non-halting user code. Davinci's inversion is
validated: dialects are a closed enum at S1; WASM extensions contribute facts
and rules, never grammar. One structural import: dialect lowerings that
synthesize identifiers (slot props, `v-for` scopes) need hygiene-style scope
tagging so synthesized names can't capture user bindings.

## Recent literature (2022–2026)

**Region IRs validated — S2 as designed.** V8 abandoned sea-of-nodes for a CFG
IR in 2025 with compile time halved
([Land ahoy](https://v8.dev/blog/leaving-the-sea-of-nodes)): graph IRs pay off
only when operations are pure and reorderable, and JS is effect-dominated. UI
templates are effect-dominated _and_ structured by construction, so
region-structured S2 is what the field converged to — and Davinci skips
RVSDG's expensive restructuring step because templates have no gotos
([RVSDG, TECS 2020](https://dl.acm.org/doi/10.1145/3391902)). One RVSDG
mechanism imported: **state edges** — S3 encodes DOM/effect ordering as
explicit dependencies, not implicit walk order, making partition and grouping
local graph queries. _Import now._

**Rendering as incremental view maintenance — the S3 theory.**
[DBSP (VLDB 2023 best paper)](https://docs.feldera.com/vldb23.pdf): every
operator has an incremental form; linear operators incrementalize for free,
non-linear ones need memoization. Mapping: a keyed `v-for` is a linear
operator whose patch plan _is_ the incremental circuit; non-linear mixes of
reactive sources are exactly where cache/memo ops belong. This derives patch
flags and SSR plans from **operator linearity** instead of ad-hoc rules, and
yields a mechanical oracle: _incremental update output ≡ from-scratch render_.
[React-tRace (2025)](https://arxiv.org/abs/2507.05234) supplies the nearest
precedent — an executable reference interpreter for a production reactive
model, validated by a conformance suite; differentially testing _optimized
codegen_ against such a reference is **our proposed extension of that
method**, not a claim of the paper. _IVM framing: import now; reference
interpreter: prototype later._

**Incrementality boundaries — the two-tier split sharpened.**
[matklad's 2026 critique of query-based compilers](https://matklad.github.io/2026/02/25/against-query-based-compilers.html):
fine-grained queries are a tax imposed by language design; locality-friendly
languages should parse in parallel and sequence only _summaries_.
[CodeQL's incrementalization (FSE 2023)](https://arxiv.org/abs/2308.09660):
fully-incremental datalog cost ~70GB RAM; the winner was hybrid — batch the
non-recursive parts, incrementalize only recursion. Import: the **per-SFC
summary (props/emits/slots types, component refs) is the only cross-file salsa
key**; template-body edits never cross the file boundary unless the summary
changes; only recursive fact groups (graph reachability, route typing,
transitive slots) are incrementalized — block-local facts recompute from
content-keyed artifacts. _Import now._

**Datalog engines.** [Ascent (CC 2022)](https://dl.acm.org/doi/abs/10.1145/3497776.3517779)
(compiled Rust macro datalog) is the only engine shape that fits — candidate
for the 2–3 genuinely recursive fact groups in the salsa tier, never the fused
path. [Glean](https://glean.software)'s schema discipline (typed, versioned,
demand-derived facts with provenance) confirms the fact-group design at scale.
_Ascent: prototype later; Glean's model: design reference now._

**Equality saturation.** Davinci's optimization space (hoist/cache/group) is
small and mostly confluent — full eqsat solves phase-ordering problems we don't
have, and binder-heavy terms (`v-for` scopes) are where e-graphs still hurt
([slotted e-graphs, PLDI 2025](https://dl.acm.org/doi/10.1145/3729326)). The
transferable shape is Cranelift's aegraph discipline: **keep placement
alternatives explicit in S3 (hoisted/cached/inline/grouped) and defer the
choice to one cost-driven extraction point** instead of committing during the
walk. _Deferred-extraction: prototype later if greedy decisions pessimize._

**Testing.** [MetaMut (ASPLOS 2024)](https://connglli.github.io/pdfs/metamut_asplos24.pdf) /
WhiteFox (OOPSLA 2024): Folio dumps make Davinci a metamorphic-testing
goldmine — semantics-preserving SFC mutations (attribute reorder, pass-through
wrappers, text-node splits) must yield S3 folios identical modulo ids; an
LLM-guided loop that reads an optimization pass and synthesizes exercising
templates layers onto the corpus harness cheaply. Alive2-style full SMT
translation validation is overkill; per-stage reference semantics +
differential folio checks are the tractable version. _Metamorphic folios:
import now; translation validation: prototype later._

**WASM component model as the external-dialect ABI.**
[WAW @ POPL 2025](https://popl25.sigplan.org/details/waw-2025-papers/4/The-WebAssembly-Component-Model):
WIT-typed, versioned interfaces; a
[practitioner write-up](https://techbytes.app/posts/wasm-component-model-plugin-architecture/)
reports large throughput gains over JSON-RPC (workload details unverified —
our own transport benchmark in plan P6-1 is the number that gates); and the
`wasm32-wasip2` core target (charter #18) means external dialects can run
out-of-process _or_ in-process under wasmtime against the same contract.
Caveat: the canonical ABI copies at boundaries — interfaces must be
coarse-grained (whole block in, surface tree out), never per-node.
_Import now — the leading transport candidate for charter #15, confirmed or
refuted by the P6-1 measurement._
