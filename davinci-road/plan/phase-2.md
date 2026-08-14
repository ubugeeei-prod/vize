# Phase 2 — Disegno and the Pass Manager (provisional decomposition)

> [!WARNING]
> Provisional: re-reviewed at phase-1 exit. IDs stable; scopes may split.
> Concreteness here is best-current-knowledge — paths/type names are the plan,
> measurements may overrule them.

## TODO index

- [ ] P2-1 `vize_davinci` core types
- [ ] P2-2 Pass manager
- [ ] P2-3 `PassObserver`
- [ ] P2-4 Folio derive + `davinci-opt` pipelines
- [ ] P2-5 `vize_disegno` S2 type family
- [ ] P2-6 S2 verifier v1
- [ ] P2-7 S1 Vue surface tree
- [ ] P2-8 S1→S2 Vue lowering
- [ ] P2-9 Core transforms as S2 passes
- [ ] P2-10 Style `v-bind()` ops
- [ ] P2-11 DOM backend on S2
- [ ] P2-12 Fused build path + walk-count instrumentation
- [ ] P2-13 Folio-after-change / `vize repro` / timing JSON
- [ ] P2-14 wasm32-wasip2 + no_std CI lanes
- [ ] P2-15 Metamorphic suite v1
- [ ] P2-16 JSX lowering re-targets S2
- [ ] P2-17 IR contract review milestone
- [ ] P2-18 Spolvero feed v1
- [ ] P2-19 DevTool protocol spike
- [ ] P2-20 Phase exit

---

**P2-1 `vize_davinci` core types.** Extend the P0-10 crate: `NodeId` (u32
newtype), side-table maps (`FxHashMap<NodeId, T>` initially; densify later),
the unified `Diagnostic` type (span + stage-of-origin + structured parts +
witness slot), `no_std + alloc`. _Accept:_ size asserts; wasip2 build; docs
per type.

**P2-2 Pass manager.** `Pipeline` as const data; pass classification enum
`{MandatoryDiagnostic, MandatoryLowering, Optional}` (SIL import); fusable/
barrier markers; `Raw<S>`/`Canonical<S>` wrapper types so only mandatory
passes convert; textual pipeline syntax `s2(a,b),s2-to-s3(c)` parsed for
`davinci-opt`. _Accept:_ unit tests over pipeline parsing/fusion grouping;
fusion computes preserved-set intersections at build time.

**P2-3 `PassObserver`.** Seven hooks (before/after pipeline, pass, analysis +
fail); observers: timing (JSON per P0-4 schema), folio printing, budget
counting, remark sink (empty until P3-13). Fusion groups reported explicitly
so timing never lies. _Accept:_ zero-cost when no observer attached (bench).

**P2-4 Folio derive + pipelines.** `#[derive(Folio)]` proc-macro (print/parse/
stable field order from the type shape — the ODS lesson: derive the mechanical
trio only); `davinci-opt --pipeline "<syntax>" --stage <s>` runs passes on a
parsed folio. _Accept:_ round-trip property test per derived type; a pass
test = folio in → pipeline → snapshot out.

**P2-5 `vize_disegno` S2 types.** Op enums (element/component/text/interp/
`ui.if{regions}`/`ui.for{binding,region}`/`ui.slot`/`ui.model{contract}`/
`vue.directive`), `ExprRef<'a> { Js(&'a Expression<'a>), Foreign(&'a
ForeignExpr<'a>) }` (Foreign = type only, charter #28), region ownership.
No `_` arms anywhere downstream. `ExprRef` gets an **owned Folio payload**
(arena references cannot persist): `Js` serializes as source slice + span and
re-parses into the arena on folio load; `Foreign` as dialect id + source +
span. _Accept:_ folio round-trip incl. `Js`/`Foreign` full-mode fixtures and
an arena-reset replay test; size asserts; exhaustive-match compile test
(adding a variant breaks a canary).

**P2-6 S2 verifier v1.** Local checks only (GHC Lint discipline): region
nesting, id resolution, expr-ref liveness, canonical-form invariants (each
documented in `folio-format.md`). Runs between passes in debug/CI via
observer. _Accept:_ verifier rejects hand-built invalid folios (fixture set);
never ships in release hot path.

**P2-7 S1 Vue surface tree.** Lossless template tree with trivia,
`Unexpected`/`Missing` structural error nodes (SwiftSyntax import),
`render(tree) == source` debug verifier; armature emits it (or a thin layer
over relief until relief splits). _Accept:_ byte-fidelity property over the
corpus (parse → render == input, including malformed fixtures).

**P2-8 S1→S2 Vue lowering.** Total function, no rollback (MLIR import);
hygiene scope-tags on synthesized identifiers (slot props, v-for scopes);
`MacroExpansionInfo`-style provenance pairs recorded. _Accept:_ every corpus
template lowers or produces a diagnostic (no panics — totality fuzz lane).

**P2-9 Core transforms as S2 passes.** Port `vize_atelier_core/src/transforms/`
one at a time: structural if/for (regions replace sibling mutation), slots
normalization, text/interp merging, hoist-static as an S2 analysis pass.
Old lane stays live (flag). _Accept:_ per-pass folio snapshots; DOM output
via old codegen unchanged (corpus).

**P2-10 Style `v-bind()` ops.** `vize_atelier_sfc`'s css-vars coordination
surfaces as S2 binding ops (charter #13). _Accept:_ facts visible in the
croquis folio; compile output unchanged.

**P2-11 DOM backend on S2.** `vize_atelier_dom` lowers S2 → codegen structure
directly; the relief codegen-node universe (`NodeType` 13–26) stops being
written by the new path. In-phase flag `VIZE_DAVINCI_DOM=legacy` for
fallback. _Accept:_ corpus DOM byte-parity on the new path; patch-flag
equivalence fixtures.

**P2-12 Fused build path.** Parse → S2 direct (S1 materialization on demand
only — formatter/autofix consumers); walk-count instrumentation via observer;
compare against the P0-3 walk baseline. _Accept:_ traversal count ≤
pre-Davinci pipeline on the fixture ladder, measured in CI.

**P2-13 Folio-after-change / repro / timing.** `--folio-after-change`
(hash-gated printing per pass), `--folio-dir`; panic handler writes
`repro.folio` (last-good stage dump + pipeline string + config) and `vize
repro <file>` replays it (charter #30); timing JSON per P0-4 schema.
_Accept:_ injected panic produces a replayable repro in a test.

**P2-14 Portability lanes.** Starts with the **`no_std` boundary audit** the
open question calls for: which dependencies genuinely support
`no_std + alloc`, the approved boundary documented, and the `wasm32-wasip2`
**core-compile lane** (davinci crates only) explicitly separated from the
full-CLI lane (which stays `std`). Then: wasip2 + `--no-default-features` CI
jobs for `vize_davinci`/`vize_disegno`; std-gated edges documented. The
existing workspace makes no `no_std` claim until this audit says so.
_Accept:_ audit doc committed; lanes green and required for the new crates
only.

**P2-15 Metamorphic suite v1.** Mutators: attribute reorder, pass-through
`<template>` wrap, text-node split/merge, whitespace-insignificant edits —
**each mutator ships an equivalence justification and exclusion predicates**,
because these are _not_ universally semantics-preserving in Vue (no
reordering across duplicate keys or `class`/`style` merge-order-sensitive
attrs; wraps only where root/slot semantics are unchanged; whitespace only
per Vue's condense rules). A mutator with no safe applicability at a site
skips that site rather than mutating it. Oracle: S2 folios identical modulo
declared normalization. _Accept:_ suite runs over matrix fixtures + a corpus
shard in CI; per-mutator justifications reviewed.

**P2-16 JSX lowering re-targets S2.** `vize_atelier_jsx` lowers to Disegno
instead of relief `RootNode`; behavior parity via existing babel-compat
oracle tests. _Accept:_ JSX corpus + `babel_compat_oracle` green on the new
path.

**P2-17 IR contract review milestone.** Checklist review against the
prior-art rules: no redundant encodings (semantic xor derived-and-cached),
no constructor folding, escape-variant (`Expr::Opaque`?) has pessimal
documented semantics, spans survive lowering. **Review point** — last cheap-
fix window before caches/DevTool depend on formats. _Accept:_ signed-off
checklist committed.

**P2-18 Spolvero feed v1.** Observer → folio directory + payload schema;
`vize_curator` inspector renders S1/S2 pages next to the existing VIR alias.
_Accept:_ playground shows the stage ladder for a compiled SFC.

**P2-19 DevTool protocol spike.** Resolve the open question (JSON-lines
stream vs files vs content-mapper-style RPC) with a working prototype against
the P2-18 feed. _Accept:_ decision recorded in devtool.md; spike code kept or
deleted deliberately.

**P2-20 Phase exit.**

- [ ] DOM corpus byte-parity on the S2 path; legacy DOM lane + flag deleted
- [ ] Traversal budget ≤ baseline in CI; verifier + metamorphic + **totality-fuzz (TS-20)** suites green
- [ ] S1/S2 folios in fixtures; `davinci-opt` pass tests in place
- [ ] wasip2/no_std lanes required; IR contract review signed off
