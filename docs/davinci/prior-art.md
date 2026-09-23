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

---

The survey continues in [Prior Art — surface toolchains, assurance practices, and literature](./prior-art-toolchains.md): Swift, GHC, OCaml Flambda2, Lean 4, and the 2022–2026 literature.
