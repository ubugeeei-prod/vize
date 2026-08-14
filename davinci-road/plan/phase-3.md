# Phase 3 — Impeto and Backend Convergence (provisional decomposition)

> [!WARNING]
> Provisional; re-cut at phase-2 exit. Suites referenced as TS-n from
> [test-suites.md](./test-suites.md).

## TODO index

- [ ] P3-1 `vize_impeto` crate + phase validator
- [ ] P3-2 Reactivity lattice fact group v1
- [ ] P3-3 S2→S3 lowering + shared partition
- [ ] P3-4 Lean reference semantics + differential runner
- [ ] P3-5 Impeto op reference doc (before optional passes)
- [ ] P3-6 Vapor backend on S3
- [ ] P3-7 VDOM patch flags from lattice facts
- [ ] P3-8 SSR thin path
- [ ] P3-9 S4 structured emitter + universal source maps
- [ ] P3-10 Try-measure-commit extraction
- [ ] P3-11 IVM oracle
- [ ] P3-12 Behavioral (sprout) runner incl. IME scripts
- [ ] P3-13 Optimization remarks + corpus remarks-diff
- [ ] P3-14 `folio-reduce`
- [ ] P3-15 Lean theorems (lattice / grouping / IVM linearity)
- [ ] P3-16 Phase exit

---

**P3-1 `vize_impeto`.** Flat id-based ops (generalizing
`vize_atelier_vapor/src/ir.rs`'s 16 variants), **explicit state edges** for
DOM/effect ordering, named phases `built → partitioned → scheduled` with a
between-pass validator (edges resolve, regions nest, effects well-scoped —
TS-27), `no_std + alloc`, folio from birth (TS-16). _Accept:_ TS-16/24/27;
size asserts.

**P3-2 Reactivity lattice v1.** Fact group classifying bindings/expressions
(static → props-stable → reactive → unstable) using the React-Compiler effect
vocabulary (`Freeze`/`Capture`/`MutateGlobal`…) as per-binding summaries over
retained oxc ASTs; escape analysis drives demotion; `provide/inject`-derived
bindings cap at `reactive` (Effekt lexical/dynamic rule). **Lattice states
and verdicts are orthogonal axes:** the four lattice states are the fact's
_value_; `proven/refuted/unknown` is the epistemic _status_ of that value —
a binding can be provenly `reactive` or unknowably classified, and rules fire
only on proven values per the assurance doctrine. _Accept:_ declarative rule spec + naive evaluator committed
(TS-34 pattern); lattice folio page.

**P3-3 S2→S3 lowering.** Total, no-rollback; static/dynamic partition
computed once here and exported as facts (SSR reads them without S3).
**Exported partition facts describe canonical S3 only**: optional passes
(P3-10 extraction) either provably preserve the partition (verifier-checked)
or trigger fact revalidation before anything downstream reads them — stale
exports are a verifier failure, not a footgun. ANF-ish
discipline: pure expressions vs effectful ops separated. _Accept:_ TS-17
pass snapshots; TS-20 totality fuzz extended to S2→S3.

**P3-4 Lean reference + differential.** `formal/impeto/` Lean package
(CI-lenient lane per charter #39): executable small-step semantics for S3 ops
under both Vapor and VDOM interpretations; runner compares compiled-output
behavior traces vs reference on S3 fixtures (TS-28). _Accept:_ runner in CI
on the fixture ladder.

**P3-5 Op reference doc.** `davinci-road/plan/impeto-ops.md`: every op's
meaning under both interpretations, written **before any optional pass
lands** (MIR anti-lesson); Lean file is the normative companion; Folio is the
concrete syntax. _Accept:_ review point — signed off; doc cross-linked from
rustdoc.

**P3-6 Vapor on S3.** `vize_atelier_vapor` lowers S2→S3→generate with full
semantic context; deletes the run-then-discard double transform
(`compile.rs`) and the duplicated directive transforms
(`transforms/{v_if,v_for,v_on,v_bind,v_model,v_show,transform_slot,transform_text}.rs`);
calls upstream `@vue/runtime-vapor` APIs only (charter #38). In-phase flag
for fallback. _Accept:_ TS-33 behavioral parity; TS-30 traces; vapor bench
improvement (the P0-3 double-transform number is the floor to beat).

**P3-7 VDOM patch flags from facts.** `patch_flag.rs` inference replaced by
lattice-fact consumption; flags become explicit S3 decisions (or S2→S4
annotations if S3 detour measures badly — decide by TS-22). _Accept:_ corpus
DOM byte-parity (TS-11 empty); patch-flag equivalence fixtures.

**P3-8 SSR thin path.** S2→S4 string-plan lowering reading partition facts;
`vize_atelier_ssr` codegen re-targets. _Accept:_ SSR corpus byte-parity
(TS-11 empty for ssr).

**P3-9 S4 emitter + source maps.** Structured span-carrying emission document
replaces `CodegenContext.code` string appends across dom/vapor/ssr; one
`SourceMapBuilder`; **SSR and Vapor emit source maps**; delete
`crates/vize_atelier_sfc/src/source_map.rs` text-matching recovery.
_Accept:_ TS-31 coverage budget — **the numeric threshold is pinned in
`budgets.toml` before this task merges**, and the text-matching recovery may
only be deleted once the new path's measured coverage ≥ the old heuristic's
measured coverage; TS-11 empty (maps are additive artifacts).

**P3-10 Try-measure-commit.** Placement alternatives (hoist/cache/inline/
group) kept explicit on S3 nodes; extraction pass performs candidates,
locally simplifies with fact approximations in scope, measures (emitted
size, reactive-edge count, update-path length), commits under an explicit
multi-metric rule — **no metric may regress beyond a per-metric ε and at
least one must improve** (correctness-adjacent metrics like reactive-edge
count are constraints, size is the objective; the ε values and ordering are
pinned in `budgets.toml` at task start from corpus distributions, ties reject
in favor of the simpler shape) — under a decrementing per-component budget;
`-O` tiers = budget constants in `budgets.toml`. _Accept:_ TS-17 snapshots of decisions; TS-32 remarks record
applied/missed; no output regression (TS-11/TS-33).

**P3-11 IVM oracle.** Incremental-update ≡ from-scratch render on the Lean
reference for keyed/unkeyed `v-for`, conditional toggles, mixed non-linear
expressions (TS-29). _Accept:_ suite green over matrix fixtures.

**P3-12 Behavioral runner.** Sprout-style: mount compiled VDOM + Vapor
against scripted prop/interaction traces in a headless DOM; **IME composition
scripts pin `ui.model` realizations** (charter #40): compositionstart →
intermediate input → compositionend, `.lazy`/`.number`/`.trim`, checkbox
arrays, select-multiple (TS-30). _Accept:_ trace equality across backends and
vs reference.

**P3-13 Remarks.** `{pass, kind: applied|missed, span, args}` structured
remarks through the observer; corpus remarks-diff job (TS-32); missed-remarks
feed C-13. _Accept:_ remarks render in Spolvero (C-5); diff job wired.

**P3-14 `folio-reduce`.** Interestingness-script driver (llvm-reduce model)
with S1-subtree deletion vocabulary; oracles composable from diagnostics /
remarks / folio content / budget breaches. _Accept:_ reduces a seeded crash
fixture to ≤ 20% size while preserving the oracle.

**P3-15 Lean theorems.** Lattice laws (classification monotonicity, join),
effect-grouping preserves dependency edges, keyed-`v-for` IVM linearity —
proved against the P3-4 semantics as they stabilize. _Accept:_ theorems in
CI-lenient lane; failures block S3-semantics changes, not unrelated PRs.

**P3-16 Phase exit.**

- [ ] Vapor: TS-33 behavioral parity green; SSR **and VDOM**: TS-11 byte-empty (P3-7 changes patch-flag derivation, so DOM parity re-gates here)
- [ ] TS-31 source-map coverage ≥ budget on all three backends
- [ ] Vapor compile bench beats the pinned double-transform floor
- [ ] TS-32 remarks-diff clean; old vapor/ssr lanes + flags deleted
- [ ] TS-27/28/29/30 all mandatory-green; TS-20 totality fuzz extended to S2→S3 green
