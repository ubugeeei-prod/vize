# Continuous Workstreams (cross-phase)

> [!NOTE]
> TODOs that never belong to one phase: they start when their substrate
> exists and run until program end. Each item names its trigger.

## Spolvero (DevTool)

- [ ] C-1 Protocol spike (starts: P2, alongside the observer API)
- [x] C-2 S1/S2 folio pages in the inspector (starts: P2-18)
      _Landed 2026-09-21: the wasm `analyzeSfc` feed carries the S1 page, the
      S2 lowering page and one S2 page per executed transform pass
      (`vize_curator::inspector::ladder_pages`, TS-52 `spolvero_ladder`), and
      the playground's Davinci tab (`playground/src/features/davinci/`)
      renders them after negotiating `schema_version`; the P2-18 rendering
      review point is now a VRT baseline (`davinci-{light,dark}.png`) plus
      the real-wasm `e2e/davinci-ladder.test.ts`. The `vize inspector`
      payload stays S1-only on purpose (it rides in share URLs; the playground
      recomputes the ladder from the same sources)._
- [ ] C-3 Pass timeline + fusion-group view from timing JSON (starts: P2-13)
      _Partial 2026-09-21: the Davinci tab shows every executed step in run
      order and marks which passes changed the folio (exact page compare).
      Open: per-pass timing and fusion groups (the profiler clock is
      `std::time::Instant`, which wasm32-unknown-unknown lacks)._
- [ ] C-4 Flame views from profiler export (starts: P0-11 data available)
- [ ] C-5 S3 pages, provenance navigation, remarks rendering (starts: P3-13)
      _S3 pages landed 2026-09-21 in the same feed: `s3` (graph),
      `s3-partition` (the new `[s3-partition-folio]` page) and `s3-values`.
      Provenance navigation landed in the Davinci tab the same day: a stage
      line highlights its authored span in the editor, and the editor cursor
      marks the narrowest stage line covering it. Open: remarks rendering
      (P3-13)._
- [ ] C-6 Fact browser incl. reactivity-lattice overlay (starts: P4-1)
      _Constraint recorded 2026-09-21: the lattice has no per-SFC producer
      yet (P3-7 evaluates it per binding kind inside DOM patch emission), so
      no `[s3-reactivity-folio]` page can be fed for a template today._
- [ ] C-7 `vize devtool` local server, editor-agnostic (starts: after C-5)
- [ ] C-8 Fresco TUI view for pass timeline/diagnostics (starts: opportunistic; Fresco itself is frozen, consuming it is allowed)
- [ ] C-9 Standing gate enforcement: every landed stage ships its folio page + Spolvero view (from P2 on)

## AI optimization loop (charter #16/#32)

- [ ] C-10 Loop harness: profile diff → candidate → gates → PR (starts: P0-4 gates exist)
- [ ] C-11 Auto-merge wiring with audit trail (auto-merged PRs carry gate evidence; starts: after C-10 has human-reviewed history)
- [ ] C-12 Sandboxing: worktree isolation + corpus-run quotas for experiments
- [ ] C-13 Missed-remarks mining as the optimization backlog (starts: P3-13) _(first version: [remarks-backlog.md](./remarks-backlog.md), mined from the TS-32 corpus)_

## Corpus operations (charter #31)

- [ ] C-14 Expansion audits at every phase boundary (surfaces the phase touches)
- [ ] C-15 Hydration/runtime cost management (sharding, caching) as the corpus grows
- [ ] C-16 Waiver-ledger stewardship: empty at every phase exit, reviewed in between

## Assurance operations (charter #21)

- [ ] C-17 Ratchet stewardship: `budgets.toml` numbers only tighten; loosening requires a charter-referenced PR
- [ ] C-18 Mutation-testing coverage expansion crate-by-crate as Davinci crates land
- [ ] C-19 FP/FN ledger triage cadence (every phase boundary at minimum)
- [ ] C-20 Fuzz-target expansion per new stage (S1 parser, folio parser, S2/S3 verifier inputs)
- [ ] C-21 Metamorphic mutator library growth (new SFC mutations as constructs land)

## Formal methods (charter #36)

- [ ] C-22 Lean CI lane maintenance (CI-lenient dependency lane per charter #39)
- [ ] C-23 Independent Lean folio checker, expanded stage-by-stage (starts: P2 folios stable)
- [ ] C-24 Theorem backlog: lattice laws → effect-grouping edge preservation → IVM linearity (starts: P3-4)

## Documentation truth (charter #45 pending)

- [ ] C-25 Architecture-doc truth passes at each phase exit (docs claim only what ships)
- [ ] C-26 Charter/open-questions hygiene: decided items become stubs; the consumption/rule-parity matrices stay current via their staleness checks
