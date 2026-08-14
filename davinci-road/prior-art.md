# Davinci — Prior Art & Imported Practices

> [!NOTE]
> Surveyed 2026-08-13 against current sources (links inline). Each entry states
> what the system does, what Davinci imports (with the concrete mapping), and
> what it deliberately does not. This page is the justification trail for
> practices referenced by the [architecture](./architecture.md) and
> [roadmap](./roadmap.md).

## rustc — MIR, Polonius, queries

**MIR / phase discipline.** MIR wins because it is the _minimal structure over
which the flagship analysis is naturally expressible_, with named phases each
carrying validated invariants (`-Zvalidate-mir`), and per-pass dump testing
that moved from full golden diffs to targeted FileCheck assertions
([mir-opt FileCheck](https://github.com/rust-lang/rust/pull/116810)).
_Import:_ Impeto gets named phases (`built → partitioned → scheduled`) with a
cheap between-pass validator. rustc's move to targeted FileCheck assertions is
adopted only _under_ the [assurance doctrine](./assurance.md): the oracle
remains the full normalized folio snapshot (exact); targeted assertions are
structural-equality supplements documenting the pass's claimed property —
rustc's churn problem is solved by printer normalization, not looser oracles. THIR's ephemeral-bridge pattern
licenses S2→S3 scratch structures that are never persisted stages.
_Anti-lesson:_ MIR optimizations were chronically unsound because MIR's runtime
semantics were never pinned down first — the S3 op reference (what each effect
means under Vapor and VDOM interpretation) is written **before** fusable passes,
with Folio as its concrete syntax. rustc's `optimized_mir` "steal" coupling is
the named counter-example for keeping phase outputs immutable.

**Polonius.** The datalog formulation remains the borrow checker's _spec_; the
shipped implementation (nightly 2026-08) is a reformulation inside NLL because
materializing fact × CFG-point relations was fatally slow
([enabling polonius alpha](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/)).
_Import:_ every fact group keeps a small declarative rule statement in
docs/tests with a naive evaluator as a **differential oracle in the corpus
harness**; implementations stay hand-written fixpoints. Design-review smell:
any fact keyed by (something × every expression position). Staged precision —
coarse location-insensitive fact first, precise analysis only where the coarse
one is inconclusive — is the template for expensive lattice analyses.

**Query system / incremental.** Red-green + fingerprint early-cutoff; _relative
spans_ so position shifts don't invalidate ([#84373](https://github.com/rust-lang/rust/pull/84373));
stable `DefPathHash` vs dense per-session `DefId`; the
[1.52.1 incident](https://blog.rust-lang.org/2021/05/10/Rust-1.52.1/) (silent
fingerprint unsoundness for years because verification was off).
_Import:_ block content keys hash **span-relative** structure (absolute
positions live in S0 side tables); stable content keys vs dense NodeIds as the
two-level naming; **incremental-vs-clean equivalence in CI from the first
salsa-backed release**. The two-tier execution split (fused CLI, salsa
resident) is exactly rustc/rust-analyzer precedent.

**salsa (0.28.x, 2026).** rust-analyzer's structure: inputs (file text, crate
graph), interned ids, **durability layers** (library inputs vs open buffers),
and **firewall queries** — small stable derived values that stop edit noise via
backdating. The warning about per-entity tracking comes from the **March 2025
port** (pre-0.28 salsa; the figures belong to that version, hardware
unspecified), from two distinct private-codebase reports:
[#19402](https://github.com/rust-lang/rust-analyzer/issues/19402) — memory
rising from **5–6 GB to ~22 GB** shortly after startup and up to **~30 GB**
in use, on a private Bazel-based codebase of ~700 crates (~2,800 including
external deps); and
[#19404](https://github.com/rust-lang/rust-analyzer/issues/19404) —
`parallel_prime_caches` going from **~26 s to ~112 s** in a different private
monorepo of ~100 workspace crates. Both were subsequently tuned down.
_Import:_ block content keys as the firewall query; durability =
`node_modules`/tsconfig high, buffers low; track at block granularity with
arena-packed stage values inside; deterministic ordering in every query result
(Folio-testable); explicit interning GC bounds.

## LLVM / MLIR

**Dialect design.** Lattner's retrospective: scaling before foundations settle
freezes accidental behavior into contracts
([What about MLIR?](https://www.modular.com/blog/democratizing-ai-compute-part-8-what-about-the-mlir-compiler-infrastructure));
the conversion framework's rollback machinery became the slowest part and is
being retired for one-shot lowering
([one-shot RFC](https://discourse.llvm.org/t/rfc-a-new-one-shot-dialect-conversion-driver/79083)).
_Import:_ stage additions and cross-stage escape hatches require charter-level
review; lowerings are total functions that fail with diagnostics, **never
rollback**; per-stage canonical form is a one-page documented doctrine whose
regression test is the Folio snapshot itself. The moment S2 grows a variant
that exists only for one input syntax, we've started MLIR's dialect-overload
problem — that's the review question for every S2 change.

**Pass manager.** Five features imported nearly verbatim from
[MLIR's pass infrastructure](https://mlir.llvm.org/docs/PassManagement/):
textual pipeline syntax (`s2(hoist-static,region-merge),s2-to-s3(...)`) as the
keystone for single-pass testing; a single `PassObserver` trait (seven hooks)
carrying timing, Folio printing, budget enforcement, and remarks at zero cost
when detached — reporting **fusion grouping explicitly so timing never lies**;
`--folio-after-change` (hash-gated printing) turning "which pass broke this"
into reading; crash reproducers = last-good folio + pipeline string, replayable
via `vize repro`; machine-readable timing (JSON) so CI gates on the traversal budget.

**`davinci-opt`.** MLIR's testing culture rests on `mlir-opt` + round-tripping
textual IR. _Import:_ Folio must **parse, not just print**, for S2/S3; a
`davinci-opt` binary reads a folio, runs a named pipeline, prints a folio.
Without round-trip, every pass test drags the full upstream pipeline — the
exact coupling MLIR's guide warns against. A `#[derive(Folio)]` proc-macro
covers the mechanical print/parse/field-order trio; verifier logic and
lowerings stay hand-written (the ODS anti-lesson: don't build an op-DSL).

**Interfaces without dyn.** OpInterface's effect — generic passes over
capability-typed ops — is reproduced statically: capability traits
(`HasRegions`, `SpanCarrier`, `Reactive`) implemented as exhaustive matches on
closed stage enums, monomorphized generic walks. The one designated `dyn` seam
is S4 emitters (per-target trait objects, cheap and right). No `_` arms on
stage enums — adding a variant must break every pass that has to handle it.

**Analysis invalidation.** LLVM's new-PM model — lazy analyses from a manager,
passes return `PreservedAnalyses` _post-hoc_, named preservation sets, plus a
debug mode that recomputes "preserved" analyses and asserts equality (absence
of which caused years of stale-analysis bugs)
([new PM](https://blog.llvm.org/posts/2021-03-26-the-new-pass-manager/)).
_Import:_ the fact engine's invalidation model, verbatim; a fused walk
preserves the intersection of members' preserved sets, computed at fusion time.

**Remarks.** LLVM's structured optimization remarks with source locations +
opt-viewer/opt-diff ([remarks](https://releases.llvm.org/13.0.0/docs/Remarks.html))
are the highest-leverage DevTool import: every decision pass emits
`{pass, kind: applied|missed, span, args}` with **structured args** (free-form
strings are unfilterable — their regret), keyed to authored SFC spans.
`remarks-diff` over the corpus catches optimization regressions without output
diffing; missed-remarks are a mined feature backlog.

**IR contract debt.** LLVM's three expensive regrets — redundant encodings
(pointee types: ~7 years to remove; typed GEP still migrating), underspecified
escape values (undef/poison), constructor-time folding (top infinite-loop
source) — were all known early and cheap to fix early
([nikic](https://www.npopov.com/2021/06/02/Design-issues-in-LLVM-IR.html)).
_Import as rules:_ every S2/S3 field is either semantic or derivable-and-cached,
never both; any `Expr::Opaque` escape variant gets pessimal documented
semantics from day one; folding happens in exactly one designated pass per
stage. One **IR contract review milestone** before DevTool/caches depend on the
formats — the last cheap-fix window.

**folio-reduce.** llvm-reduce's design (dumb driver, sovereign interestingness
script, IR-aware reduction vocabulary) is feasible and nearly free for Davinci:
reduce the SFC via S1 subtree deletion (always re-printable), oracles composed
from diagnostics, remarks, Folio content, and budget violations — all
infrastructure that exists for other reasons
([llvm-reduce](https://llvm.org/docs/CommandGuide/llvm-reduce.html)).

## React Compiler (v1.0, 2025)

The closest prior art to the reactivity lattice — an existence proof that
reactivity is inferable from _unannotated_ JS
([release](https://react.dev/blog/2025/10/07/react-compiler-1)).

_Import:_

- The **effect vocabulary** from its aliasing model
  ([MUTABILITY_ALIASING_MODEL](https://github.com/facebook/react/blob/main/compiler/packages/babel-plugin-react-compiler/src/Inference/MUTABILITY_ALIASING_MODEL.md))
  is the lattice's missing mutability half: `Freeze` ≈ props-stable, `Capture`
  into a watcher/closure propagates reactivity, `MutateGlobal`/`Impure` force
  unstable. Implemented as per-binding effect summaries over oxc ASTs in the
  semantic engine.
- **Range grouping = effect partitioning**: disjoint-set over overlapping
  mutable ranges ("values that mutate together") is the same algorithm for
  Vapor effect grouping and VDOM patch-flag regions.
- **Granular bailout**: their unit is the function; ours is the binding/block —
  unclassifiable constructs degrade to `unstable` with conservative codegen for
  that block only, never failing the SFC.
- **One analysis, two surfaces**: their eslint rules are compiler validation
  passes re-surfaced — the argument that lattice facts feed lint and codegen
  from one source.
- **Testing wholesale**: fixture-first snap workflow (golden emitted code +
  per-pass `--debug` dumps) plus a _sprout_ equivalent — mount compiled Vapor
  and VDOM outputs against scripted prop/interaction sequences and diff
  observable behavior. This is the behavioral-equivalence tier the corpus
  currently lacks.

_Anti-lessons:_ batch-only, nothing to learn on incrementality; don't run CFG
inference where template syntax already answers the question (reserve heavy
inference for setup-scope bindings); cap memoization granularity — tracking
overhead can exceed recomputation ("good enough" beats maximal).

## MoonBit

_Import:_

- **`.mbti` interface firewalls** ([virtual packages](https://www.moonbitlang.com/blog/virtual-package)):
  split every stage artifact key into _interface hash_ (exported names, types,
  reactivity classes) vs _body hash_; dependents key on the interface hash so
  body-only edits never cascade. Interface facts are generated from S2 and
  diffed structurally — the mechanism that makes block granularity pay.
- **Fault-tolerant analysis**: semantic analysis proceeds past errors,
  producing facts for whatever is well-formed — required behavior for the
  resident tier.
- **MoonBit-as-expression-dialect, de-risked**: vendor a pinned wasm build of
  `moonc` — no user-installed toolchain, version pinned as part of the fact
  cache key. Caveat the source supports less than we'd like: the
  [documented launchers](https://www.moonbitlang.com/blog/moonbit-wasm-toolchain)
  are Node.js-based and the toolchain requires a **wasm-gc-capable runtime**;
  standalone execution under wasmtime is _our_ plan, not MoonBit's docs — the
  P6-4 spike must verify the artifact, its imports, and wasm-gc support (or
  fall back to a Node sidecar behind the same capability boundary).
  The projection is a generated `.mbti` binding environment + projected `.mbt`
  bodies, structurally identical to the virtual-TS/Corsa path.

_Anti-lessons:_ MoonBit's speed is partly language design (acyclic DAG,
explicit interfaces) — Vue/JS graphs are cyclic, so interface firewalls need
conservative widening; no cross-module WPO chasing in S4 (open-world JS); no
documented stable moonc API — wrap the CLI surface behind a capability.

## Unison

The ceiling proof for content-addressed artifacts: hash-identified definitions
make parse/typecheck results **permanently** cacheable and renames metadata
([the big idea](https://www.unison-lang.org/docs/the-big-idea/)).
_Import as disciplines, not architecture:_

- **Identity excludes presentation** — content keys hash normalized structure
  with spans externalized; formatting edits produce identical keys; the normal
  form is versioned (schema version inside every key).
- **Honest inputs** — a fact group's key covers _all_ inputs including ambient
  ones (tsconfig, moonc version, env); an undeclared input is a
  cache-corruption bug, not a perf bug. Unison's abilities are the typed
  version of our input manifests.
- **Hashes stay invisible** — humans see stable block ids and names;
  content keys are validity checks only, never user-facing.

_Anti-lesson:_ Unison's pain comes from making the database the source of
truth. Davinci inverts it: source files are truth, every artifact is a
reconstructible cache — which is why salsa stays resident-tier only.

## Effekt

Honest verdict: mostly analogy, two real imports.

- **Lexical vs dynamic scoping as an analysis boundary**: watchers/computed/
  lifecycle bind lexically to setup scope — statically resolvable;
  `provide/inject` is the dynamically-scoped exception, which is _why_ inject-
  derived bindings can never classify above `reactive` without whole-app facts.
  Encoded as a lattice rule, not a heuristic.
- **Escape = visible degradation**: second-class-by-default capabilities map to
  escape analysis as the lattice-demotion mechanism (a ref escaping setup via
  store/return/closure drops its class) — convergent with React Compiler's
  `Capture`/`CreateFunction` from the opposite direction, decent evidence the
  mechanism is right.

_Anti-lesson:_ Effekt's own retreat from its three-paper IR pipeline
([evolution](https://effekt-lang.org/evolution)) argues S3 analyses stay boring
dataflow (abstract interpretation), not a typed effect calculus, however
tempting the lattice-as-effect-system framing is. Effect typing requires
annotated cooperative source we cannot demand; evidence-passing machinery has
zero transfer (no user-visible handlers in Vue).

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
