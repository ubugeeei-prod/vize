# Phase 2 — Disegno and the Pass Manager

> [!NOTE]
> **Re-cut 2026-08-17 at the phase-1 exit**, per the [plan README](./README.md)'s rule: a provisional task cannot be picked up for implementation until it carries the full contract. The compressed blocks are now full tasks in [phase-2-tasks.md](./phase-2-tasks.md) (Deliverable / Steps / Acceptance / Deps / Non-goals); IDs are stable, two tasks split with sub-IDs. **Thesis:** phase 1 made the substrate honest — one pooled arena, expressions parsed once, 8-byte spans, no owned strings in nodes. Phase 2 spends that substrate on a **semantic IR (S2, Disegno) and a pass manager**, moves the **DOM backend** onto it as the first strangler target, and turns traversal count from an aspiration into a gated, machine-independent number. What phase-1 measurement changed relative to the provisional text is listed below — this is the record that the re-cut re-cut something.

**The per-task contracts live in [phase-2-tasks.md](./phase-2-tasks.md)** — Deliverable / Steps / Acceptance / Deps / Non-goals for all 22 tasks. This file keeps the phase-level record: what the re-cut changed, what phase 1 carried in, the TODO index that links into those contracts, and the exit gate. The two are separate files because the contracts alone exceed the repository's 350-line source-length budget (`tools/moon/cmd/source_file_lengths --max-lines 350`), which plan files are not exempt from.

## What the re-cut changed

Each item is a scope or design change forced by something phase 1 measured or landed, not a reformat.

1. **`Drop`-free is a hard constraint on every S2 type, and it is already enforced.** P1-10 collapsed `vize_carton::{Box, Vec}` onto `oxc_allocator::{Box, Vec}`, whose const assertion rejects `Drop` payloads — and that assertion caught two real violations during P1-10 (`Vec<'a, TextPart>`, `Vec<'a, ArtVariant>`), both fixed at the source rather than waived. P2-1 and P2-5a therefore state Drop-freedom as a construction rule with the container assertion as its enforcement, and carry the `grep`-zero-for-`impl Drop` acceptance P1-10 established.
2. **`ExprRef` must answer the retained-`None` classes — and that is why P2-5 splits.** The provisional two-variant enum assumed every expression has a retained oxc AST. P1-5 measured otherwise: `SimpleExpressionNode::js_ast: Option<JsExpression<'a>>` is `Some` only when the content parses as **one complete** TS expression covering the whole text; v-for values, v-on statement bodies, nesting-guard-refused text and invalid text are `None`, and P1-9 measured **11.73%** of `rewrite_expression` calls on the corpus landing in exactly those classes. A two-variant `ExprRef` has nowhere to put them. **P2-5b** is now a task whose deliverable is the decision, with the escape variant's pessimal semantics required from day one (the prior-art LLVM `undef`/`poison` rule).
3. **Phase-2 targets are pinned at phase start, by a task that owns it — and there is no walk baseline to pin against yet.** P1-13 could not tick "compile bench improvement ≥ target pinned at phase start" because no target and no phase-start baseline ever existed. Worse for this phase: the provisional P2-12 said "compare against the P0-3 walk baseline" — **P0-3 recorded expression re-parse counts, never walk counts**, and the `[traversal]` section of `budgets.toml` is empty and reserved. **P2-12 splits**: `P2-12a` is a phase-start task with no dependencies that records the pre-S2 walk baseline with its own probe (the P0-3 `expr_parse_probe` precedent), fills `[traversal]`, and pins the phase-2 target before the work that could bias it; `P2-12b` carries the original fused-build-path scope and is gated by it.
4. **Alloc counts are the enforced ratchet, so every new bench in this phase lands with its measured count.** P1-13 fixed `bench-compare.mjs` to gate `budget.allocs` exactly and independently of the wall side (machine-independent), and a seeded `0` now fails loudly. Every task here that adds a bench (P2-3's zero-cost check above all) carries that as an acceptance clause. Wall times remain report-only until the Blacksmith recording, which is P0-4's still-open pending and **not** phase 2's to close.
5. **The verification patterns are named house patterns now, not inventions.** Where a task replaces a live lane it reuses the P1-6/P1-7/P1-9 shape by name: a `#[cfg(any(test, feature = "davinci-differential"))]` dual-run comparator inside the migrated read, process-global counters, a plain-suite coverage witness that fails on a cfg regression, and a corpus-runnable entry with an exact-pinned comparison count. Zero divergence required; divergence is investigated, never averaged (TS-25).
6. **Counter and budget laws are pinned by ordinary integration tests.** P1-5 and P1-7 landed their laws as plain `#[test]` binaries (`crates/vize_armature/tests/davinci_expr_parses.rs`, `crates/vize_atelier_{dom,ssr,vapor}/tests/davinci_expr_reparse_floor.rs`) precisely so they run inside the default `cargo test --workspace` CI job rather than a feature-gated lane. P2-3's fusion-group law and P2-12b's walk law follow that shape.
7. **Node-size asserts must carry `#[cfg(target_pointer_width = "64")]`, and P2-14 is the reason.** Every node-size assert in `vize_relief` already carries the guard, with the rationale recorded at `crates/vize_relief/src/relief/elements.rs:31-36` ("the wasm32 build is 32-bit"). P2-14 makes a `wasm32-wasip2` lane **required** for the new crates, so an unguarded pointer-dependent assert in `vize_davinci` or `vize_disegno` would break that lane by construction.
8. **`SourceLocation` is 8 bytes and `Position` no longer exists**, so P2-1's `Diagnostic` keys on `vize_carton::Span` and derives line/column only at rendering time. Diagnostic message text stays **owned** — the deliberate P1-10 exception (`CompilerError::message`) — because P1-11's arena/cache contract requires anything crossing a compile boundary to be owned, enforced there by `'static` assertions on every crossing type.
9. **Folio already exists; P2-4 extends it rather than defining it.** P0-10 landed `trait Folio { print / parse }` with `FolioMode::{Full, Display}` and mode-explicit round-trip laws (`crates/vize_davinci/src/folio.rs:81`), `CroquisFolio`, and `crates/vize_davinci/src/bin/davinci-opt.rs` whose usage is today `--roundtrip <file> [--stage croquis]` with `--roundtrip` **required**. `#[derive(Folio)]` must generate that trait's exact shape, and `--pipeline` extends that binary's argument parser (making `--roundtrip` and `--pipeline` alternatives rather than one mandatory flag).
10. **Three provisional citations were wrong against the tree and are fixed here.** The timing observer's schema is P0-11's [`profile-export.schema.json`](./profile-export.schema.json) (TS-15), not "the P0-4 schema" — P0-4 is `budgets.toml`. The transform directory `crates/vize_atelier_core/src/transforms/` is reached through the Rust module **`vize_atelier_core::steps`** (`#[path]` attributes in `crates/vize_atelier_core/src/steps.rs`), and the enter/exit sibling-mutation driver P2-9 replaces lives in the sibling `src/transform/` (singular), attached from `lane.rs`. `vize repro` does **not** exist — P2-13 adds a command module and a `crates/vize/src/cli.rs` variant, it does not extend one.
11. **P2-11's flag is not a fallback, it is an unfinished deletion with an owner.** P1-13 recorded that phase 1 introduced no production fallback flag at all, and that the surviving old paths are unfinished deletions — enumerated in its gate with blockers. `VIZE_DAVINCI_DOM=legacy` is kept (charter #26 licenses it while the phase is live) but the exit gate names its deletion explicitly, and the differential lane, not the flag, is what carries the risk.
12. **Matrix fixtures do not exist yet.** P2-15's oracle runs "over matrix fixtures"; `tools/davinci/matrix-gen.mjs` defaults to `tests/fixtures/davinci-matrix/`, which **is not in the tree** (P0-12 landed the generator, not the fixtures). Committing the generated plane is now a step of P2-15 rather than an assumption.

## Carried from phase 1

Phase 1 exited with named blockers. Each is stated here with the phase-2 tasks it touches; none of them is silently absorbed.

| Phase-1 residue                                                                                                                                                                                                                         | Phase-2 tasks affected | How                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **P1-8 scanner split undeleted** — byte scanner vs AST walk disagree on 3.85% of corpus identifier extractions; blocked on the waiver decision ([#4365](https://github.com/ubugeeei-prod/vize/issues/4365), `scanner-parity-report.md`) | P2-5b                  | P2-5b defines the expression-dialect capability contract whose "enumerate referenced bindings" duty is where the single implementation would live. It must **not** encode either waiver resolution; it records the dependency and leaves #4365 to the maintainer. Phase 2 does not unblock P1-8. |
| **`transform_expression/reparse.rs` legacy chain alive** — 153 lines serving the 11.73% non-admitted rewrites                                                                                                                           | P2-5b, P2-9            | Shrinking the class needs a wider retained contract, which is exactly P2-5b's decision. P2-9 **measures** whether region-structured lowering shrinks it, using the existing `retained::differential` counters. Deletion is not a phase-2 deliverable.                                            |
| **No committed bench baseline reports; no phase target ever pinned**                                                                                                                                                                    | P2-12a                 | P2-12a pins the phase-2 target and the phase-start rev at phase start, before P2-9 merges. Recording Blacksmith wall baselines stays P0-4's open pending.                                                                                                                                        |
| **`budgets.toml [traversal]` empty, reserved "Populated by P2-12"**                                                                                                                                                                     | P2-12a, P2-12b         | P2-12a fills it from a measured pre-S2 baseline with the same machine-independence reasoning that made the alloc gate exact; P2-12b is gated by it (TS-22).                                                                                                                                      |
| **`davinci-differential` lanes live "for one release"**                                                                                                                                                                                 | P2-9, P2-11, P2-16     | The phase-2 migrations arm the same lanes. Their retirement condition is restated at the exit gate rather than expiring silently.                                                                                                                                                                |

**Registry maintenance this phase owes** [`test-suites.md`](./test-suites.md), since a gate naming an unregistered suite is a plan bug: TS-22's _From_ column still reads `P2-12` and becomes `P2-12b`; TS-25's instance list (`P1-6/7 identifiers, P1-8 scanner, P4 projection`) gains the phase-2 instances P2-9, P2-11 and P2-16; and **the feed P2-18 produces has no registered suite at all** — P2-18 must add the entry in its own PR rather than cite an invented id.

**Corpus operations note** (applies to every TS-11 run in this phase): corpus sweeps re-materialize `node_modules` inside fixture projects and stale ones corrupt hashes, so a run starts from clean fixtures (`git submodule status` all-clean, no `node_modules`) — see `corpus-baseline-notes.md` "Re-record 2". The phase-1 exit gate's recorded recipe is `node tools/davinci/corpus-diff.mjs --shards 2 --timeout-ms 600000`; the default 4-shard parallelism brushes the per-run timeout on the typechecker lane (soybean-admin), which is why the shard count is halved and the timeout raised.

## TODO index

Each ID links to its contract in [phase-2-tasks.md](./phase-2-tasks.md); what a landed task measured, decided and left open goes in [phase-2-records.md](./phase-2-records.md), one file per task. This index is the navigational entry point for the phase and the place a task's box gets checked — never before the PR that satisfies its acceptance criteria.

- [x] [P2-1](./phase-2-tasks.md#p2-1--vize_davinci-core-types) `vize_davinci` core types (`NodeId`, side tables, `Diagnostic`) — landed 2026-08-19; `NonZeroU32` niche, sparse-only side table with the densification trigger written down, owned-message `Diagnostic` with the P4-6 witness slot ([record](./phase-2-records/p2-1.md))
- [x] [P2-2](./phase-2-tasks.md#p2-2--pass-manager) Pass manager (const pipelines, classification, fusion) — landed 2026-08-19; every planning query is a `const fn`, both laws are compile errors and both were proven by compiling a violation ([record](./phase-2-records/p2-2.md))
- [x] [P2-3](./phase-2-tasks.md#p2-3--passobserver) `PassObserver` + the four in-tree observers — landed 2026-08-19; static dispatch so the un-observed path has no check at all, one profile span per walk rather than per pass, zero cost pinned by an alloc-identical bench pair ([record](./phase-2-records/p2-3.md))
- [x] [P2-4](./phase-2-tasks.md#p2-4--folio-derive--davinci-opt-pipelines) `#[derive(Folio)]` + `davinci-opt --pipeline` — landed 2026-08-20; the derive owns the mechanical trio only (`CroquisFolio` measured as unable to move, `BudgetObserver` is the first derived page), `--roundtrip`/`--pipeline` are alternatives with the roundtrip mode pinned byte-identical over the 14 P0-10 fixtures, TS-17 established with no-op bodies until P2-9 ([record](./phase-2-records/p2-4.md))
- [ ] [P2-5a](./phase-2-tasks.md#p2-5a--vize_disegno-s2-op-and-type-family) `vize_disegno` S2 op and type family
- [ ] [P2-5b](./phase-2-tasks.md#p2-5b--exprref-contract-including-the-retained-none-classes) `ExprRef` contract incl. the retained-`None` classes
- [ ] [P2-6](./phase-2-tasks.md#p2-6--s2-verifier-v1) S2 verifier v1
- [ ] [P2-7](./phase-2-tasks.md#p2-7--s1-vue-surface-tree) S1 Vue surface tree
- [ ] [P2-8](./phase-2-tasks.md#p2-8--s1s2-vue-lowering) S1→S2 Vue lowering
- [ ] [P2-9](./phase-2-tasks.md#p2-9--core-transforms-as-s2-passes) Core transforms as S2 passes (marked series)
- [ ] [P2-10](./phase-2-tasks.md#p2-10--style-v-bind-ops) Style `v-bind()` ops
- [ ] [P2-11](./phase-2-tasks.md#p2-11--dom-backend-on-s2) DOM backend on S2
- [x] [P2-12a](./phase-2-tasks.md#p2-12a--phase-start-baselines-and-pinned-targets) Phase-start baselines and pinned targets — landed 2026-08-19 at rev `232870a8`; DOM/SSR/Vapor ladder pinned in `[traversal]`, `[target.phase-2]` set, `walk-baseline.md` committed. One clause carried: the corpus `--check` is not evaluable by CI or a working tree ([record](./phase-2-records/p2-12a.md))
- [ ] [P2-12b](./phase-2-tasks.md#p2-12b--fused-build-path--walk-count-instrumentation) Fused build path + walk-count instrumentation
- [ ] [P2-13](./phase-2-tasks.md#p2-13--folio-after-change-vize-repro-timing-json) Folio-after-change / `vize repro` / timing JSON
- [ ] [P2-14](./phase-2-tasks.md#p2-14--no_std-boundary-audit--wasm32-wasip2-lanes) `no_std` boundary audit + wasm32-wasip2 lanes
- [ ] [P2-15](./phase-2-tasks.md#p2-15--metamorphic-suite-v1) Metamorphic suite v1
- [ ] [P2-16](./phase-2-tasks.md#p2-16--jsx-lowering-re-targets-s2) JSX lowering re-targets S2
- [ ] [P2-17](./phase-2-tasks.md#p2-17--ir-contract-review-milestone) IR contract review milestone
- [ ] [P2-18](./phase-2-tasks.md#p2-18--spolvero-feed-v1) Spolvero feed v1
- [ ] [P2-19](./phase-2-tasks.md#p2-19--devtool-protocol-spike) DevTool protocol spike
- [ ] [P2-20](./phase-2-tasks.md#p2-20--phase-exit) Phase exit

## Davinci describes the shipped pipeline — and cannot yet consume it (2026-08-19)

Recorded here rather than inside a task because it is a phase-level fact and no
single task owns it.

`crates/vize_davinci/src/legacy_plan.rs` declares the three backends' template
traversals as [`Pipeline`] const data — `DOM`, `SSR` and `VAPOR`, each
`transform` followed by that backend's own descent, every pass a mandatory
barrier because that is what the pipeline is today. Each backend's
`tests/davinci_walk_baseline.rs` asserts the walks the P2-12a probe counted
equal `Pipeline::group_count()` of its plan on every ladder fixture, so a plan
that drifts from the compiler it describes fails — proven by planting a
one-pass `DOM` and watching `dom small: the measured walks disagree with
legacy_plan::DOM`. A description nothing checks is a comment.

### The constraint this uncovered, which P2-11 has to answer

The declaration was written in `vize_atelier_core` first, as a normal
dependency, on the reasoning that Davinci "shipping" was the point. **The
release gate rejected it**, correctly:

    vize_atelier_core cannot depend on deferred workspace crate vize_davinci
    — tests/tooling/moonbit-publish-crates.test.ts

`vize_atelier_core` is published to crates.io; `vize_davinci` is
`publish = false`, as `vize_disegno` will be (P2-5a's contract states it). A
published crate carrying an unresolvable dependency cannot be published at all.

This is **not** a packaging detail to work around. It is a hard precondition on
the strangler plan: **P2-11 cannot move the DOM backend onto S2 while the
Davinci crates are unpublished**, because `vize_atelier_dom` is published and
would have to depend on `vize_davinci` and `vize_disegno` at runtime, not just
in tests. The options — publish the Davinci crates (and accept semver gating on
an IR still in flux), fold them into an already-published crate, or gate the S2
path behind a feature whose dependencies are optional — are a program decision,
and P2-11's contract does not currently name it.

Meanwhile the plans live in `vize_davinci` and the backends read them from
their **dev-dependencies** — the same shape `davinci_harness` already uses to
instrument published crates — so Davinci is exercised against the real pipeline
without entering anyone's release graph. What that buys is unchanged: P2-11's
target is measurable against a declaration rather than prose
(`dom_walks_max = 1` is exactly the claim that `DOM` becomes one group), and
P2-12b's swap is mechanical because the budget observer already counts what the
probe counts.

---

## Exit gate (machine-checkable)

- [ ] **DOM corpus byte-parity on the S2 path, waiver ledger empty** — `node tools/davinci/corpus-diff.mjs --surface compiler --shards 2 --timeout-ms 600000` from clean fixtures: zero gating drift with scope proof matching the project manifest (TS-11)
- [ ] **Legacy DOM lane and its flag deleted** — grep zero for `VIZE_DAVINCI_DOM` and for the old DOM codegen entry; the transform lane flag likewise (charter #26)
- [ ] **Traversal budget gated at or below the recorded pre-S2 baseline** — `budgets.toml [traversal]` populated by P2-12a and enforced exactly on the fixture ladder in CI (TS-22), with the walk law pinned by an ordinary integration test in the default `cargo test --workspace` lane
- [ ] **Verifier, metamorphic, totality-fuzz and S1-fidelity suites green and required** — TS-18, TS-21, TS-20, TS-19
- [ ] **S1/S2 folios in fixtures; `davinci-opt` pass tests in place** — TS-16 byte-exact in `Full` mode per derived type, TS-17 with at least one full normalized folio snapshot per landed pass
- [ ] **wasip2 and `no_std` lanes required for `vize_davinci` + `vize_disegno`; boundary audit committed** — TS-24
- [ ] **IR contract review signed off** — P2-17's checklist committed with the mechanical half landed as tests (review point: the judgement half)
- [ ] **Phase-2 target pinned at phase start was met, or the miss is recorded with its blocker** — measured against `budgets.toml`'s P2 target table and the phase-start rev recorded by P2-12a. This line exists because P1-13 could not tick its equivalent; it may be ticked as a recorded miss only if the target was pinned before P2-9 merged
- [ ] **Every bench added in this phase carries its measured alloc count** — `node tools/davinci/bench-compare.mjs` reports zero breaches and zero seeded-`0` entries (TS-10; a seeded `0` fails loudly since P1-13)
- [ ] **Differential lanes green and their retirement condition restated** — TS-25 zero divergence for P2-9, P2-11 and P2-16, with the phase-1 lanes' "for one release" life either honoured or explicitly re-dated
- [ ] **Standing gates held throughout** — TS-1..9 unchanged, TS-12 matrices current, TS-13 assertion lint clean under its allowlist, TS-14 mutation scores held for the new crates, TS-15 profiler export validating, TS-26 counter law still pinned
- [ ] **Corpus waiver ledger empty and the phase-boundary expansion audit done** — C-14, C-16
