# Phase 2 — Task contracts, P2-15 through P2-20

> [!NOTE]
> Continuation of [phase-2-tasks.md](./phase-2-tasks.md), split under the
> 350-line source budget when the wave of Landed headers crossed it. Same
> authority, same format; the TODO index in [phase-2.md](./phase-2.md) links
> each task to whichever file holds its contract.

## P2-15 — Metamorphic suite v1

**Landed 2026-08-21** — full record: [phase-2-records/p2-15.md](./phase-2-records/p2-15.md).

**Deliverable:** the mutator suite with per-mutator equivalence justifications — because these mutations are _not_ universally semantics-preserving in Vue.

**Steps:**

- [x] Mutators: attribute reorder, pass-through `<template>` wrap, text-node split/merge, whitespace-insignificant edits
- [x] **Each mutator ships an equivalence justification and exclusion predicates**: no reordering across duplicate keys or across `class`/`style` merge-order-sensitive attributes; wraps only where root and slot semantics are unchanged; whitespace only within Vue's condense rules
- [x] A mutator with no safe applicability at a site **skips** that site rather than mutating it, and the skip is **counted** — a suite that silently degenerates to zero mutations must fail, the scope-proof discipline TS-11 established
- [x] Oracle: S2 folios identical modulo declared normalization (the `folio-format.md` rules), compared as full normalized artifacts
- [x] Commit the matrix fixture plane. `tools/commands/davinci/matrix-gen.rs` defaults to `tests/fixtures/davinci-matrix/`, which **is not in the tree** — P0-12 landed the deterministic generator with a `--check` staleness mode but no committed fixtures. Commit the element-kind × directive plane and wire `--check` into `tests/tooling/davinci-matrices.test.ts`
- [x] Runs over the matrix fixtures **and** a corpus shard in CI

**Acceptance:** TS-21 established — the suite runs in CI over both sources with a scope proof (mutations applied and skips counted; a zero-mutation run fails); TS-12 green for the newly committed fixtures with the staleness check demonstrably failing on an injected edit; TS-13. **Review point:** the per-mutator equivalence justifications — an unjustified mutator is an oracle asserting a wrong expected value, which assurance §4 calls worse than no assertion. **Deps:** P2-5b, P2-8. **Non-goals:** S3 folio equivalence (phase 3); mutators needing semantic facts to decide applicability (phase 4); `folio-reduce` (P3-14); mutating the corpus submodules in place — copies only, the P0-13 convention.

## P2-16 — JSX lowering re-targets S2

**Deliverable:** `vize_atelier_jsx` lowering to Disegno instead of relief, which is the neutral core's first real fairness test.

**Steps:**

- [ ] `lower_source` at `crates/vize_atelier_jsx/src/lib.rs:206` — signature `lower_source<'a>(bump: &'a Allocator, allocator: &oxc_allocator::Allocator, source, lang)` — produces S2 rather than a relief `RootNode`; the crate-private `lower_source_with_compat` (`lib.rs:229`) follows
- [ ] Record whether the JSX hot path's deliberate bypass of `MarkupDocument::from_jsx` can now go. That bypass exists because Relief is Vue-shaped — it is the symptom the neutral core is supposed to remove, so its survival or removal is the honest fairness measurement
- [ ] Differential lane in the house shape for the JSX path

**Acceptance:** the babel-compat oracle green on the new path — `cargo test -p vize_atelier_jsx` (`babel_compat_oracle`), TS-6, with the nine committed snapshots unchanged; the JSX corpus projects' rows in TS-11 empty; differential lane zero divergence (TS-25); TS-1, TS-13. **Deps:** P2-11. **Non-goals:** rule-corpus fairness convergence (phase 4, TS-39) — this task _measures_ the gap, it does not close it; Svelte/Solid input dialects; deleting the relief JSX lowering, which Patina still consumes until it re-bases in phase 4.

## P2-17 — IR contract review milestone

**Deliverable:** a signed-off checklist — the last cheap-fix window before caches, Spolvero and external consumers depend on the S2 format.

**Steps** — the checklist, against the prior-art rules imported from LLVM's three expensive regrets:

- [ ] **No redundant encodings**: every S2 field is semantic **xor** derived-and-cached, never both (the pointee-type regret: ~7 years to remove)
- [ ] **No constructor-time folding**: folding happens in exactly one designated pass per stage (the top infinite-loop source)
- [ ] **The escape variant has pessimal documented semantics** from day one — P2-5b's decision is reviewed here against the `undef`/`poison` regret
- [ ] **Spans survive lowering**: every S2 op traces to an authored SFC span
- [ ] **`schema_version` on every agent-visible artifact** (devtool.md's data layer requires it: folio format, profile export, remark and fact-table schemas) so Spolvero negotiates and refuses mismatches loudly
- [ ] **Provenance survives failure**: partial S2 kept on error (P2-8's commitment, verified here)

**Acceptance:** the signed-off checklist committed. The mechanical half is machine-checked and must land as tests, not prose: a corpus-wide assertion that every S2 op's span resolves into its authored SFC, and a folio-level assertion that `schema_version` is present and negotiated. **Review point** for the judgement half — this milestone exists precisely because these are the cheap fixes that become expensive once formats have consumers. **Deps:** P2-11, P2-12b, P2-13. **Non-goals:** S3 contracts (P3-5's op reference does the same job one stage later); freezing the format for external consumers, which is phase 6's contracts GA; a stability guarantee — charter #23 keeps internal formats free to break until then.

## P2-18 — Spolvero feed v1

**Landed 2026-08-21** — full record: [phase-2-records/p2-18.md](./phase-2-records/p2-18.md).

**Deliverable:** the observer's folio output as a consumable feed, rendered in the existing inspector.

**Steps:**

- [x] P2-3's folio-printing observer writes a folio directory with a payload schema carrying `schema_version` (devtool.md's data-layer requirement)
- [x] `vize_curator`'s inspector renders S1/S2 pages — `crates/vize_curator/src/inspector/payload.rs` (`InspectorPayload`, `build_payload`, `serialize_payload`) — next to the existing croquis alias. The alias itself lives in `crates/vize_vitrine/src/wasm/analyze.rs:312-315`, which carries both the deprecated `vir` key and nested `folio.croquis` (P0-10 corrected the location; the inspector payload never carried it)
- [x] **Registry gap to close in this task:** [`test-suites.md`](./test-suites.md) has no suite covering the Spolvero feed payload. Add one there in the same PR — the registry is the source of TS-ids and a gate naming an unregistered suite is a plan bug, so this task must not invent an id here

**Acceptance:** the feed payload validates against its committed schema, gated by the newly registered suite; the croquis alias keeps working byte-identically (`folio.croquis` and `vir` both present). **Review point:** that the playground actually shows the stage ladder for a compiled SFC — a rendering claim no CI job evaluates today, which is why it is marked rather than dressed up as a gate. TS-3, TS-13. **Deps:** P2-4, P2-3. **Non-goals:** the `vize devtool` local server (C-7); the transport decision (P2-19); provenance navigation and remarks rendering (C-5, needs P3-13); the Fresco TUI (C-8).

## P2-19 — DevTool protocol spike

**Landed 2026-08-21** — full record: [phase-2-records/p2-19.md](./phase-2-records/p2-19.md).

**Deliverable:** the open question closed with a working prototype, and the spike disposed of deliberately.

**Steps:**

- [x] Prototype the three candidates against the P2-18 feed: JSON-lines stream, served files, or a content-mapper-style JSON-RPC (`vize content-mapper` is the existing precedent for the last)
- [x] Evaluate against the three consumers devtool.md names — browser playground (`vize_vitrine` wasm), local server, and `--format agent` output under `vize_doctor::ai_context` budgeting — plus the `schema_version` negotiation requirement
- [x] Record the decision in [`devtool.md`](../devtool.md) and convert the `open-questions.md` "DevTool protocol" entry into a decided stub pointing at it, per that document's own convention

**Acceptance:** decision recorded in `devtool.md`; the open-questions entry is a stub; the spike code is either kept (with tests and a home) or deleted, and the PR says which and why. **Review point** — a transport choice is a judgement, and "spike code left lying around" is the failure mode this acceptance names. **Deps:** P2-18. **Non-goals:** implementing the chosen transport at production quality (C-7); the JS plugin API shape (charter #29, phase 4/5 spike); authentication or remote access.

## P2-20 — Phase exit

**Deliverable:** the exit gate in [phase-2.md](./phase-2.md), evaluated and recorded there, in phase 0's and phase 1's manner: **a line is ticked only when it is satisfied, an unticked line names its blocker, and no line's wording is softened to make it tickable.**

**Steps:**

- [ ] Evaluate every line of the exit gate in [phase-2.md](./phase-2.md) and record the evidence inline
- [ ] Delete the in-phase old paths (P2-9's transform lane flag, P2-11's `VIZE_DAVINCI_DOM=legacy`) or record each as an unfinished deletion with an owner and an issue — charter #26's fix-forward switch happens here
- [ ] Restate the retirement condition for the `davinci-differential` lanes (phase 1's and phase 2's), which are written to live "for one release"
- [ ] Re-bench the phase-start rev and this tree, compare against the P2-12a target, and record the result — including a miss, if it is one
- [ ] Corpus waiver ledger reviewed and empty (C-16)

**Acceptance:** the exit gate in [phase-2.md](./phase-2.md), with every line either ticked with its evidence or carrying a named blocker. **Deps:** all of P2-1..P2-19. **Non-goals:** re-cutting phase 3 — that is phase 3's own re-cut at this exit, per the plan README; closing P0-4's Blacksmith pending; unblocking P1-8.
