# Continuous Workstreams (cross-phase)

> [!NOTE]
> TODOs that never belong to one phase: they start when their substrate
> exists and run until program end. Each item names its trigger.

## Spolvero (DevTool)

- [ ] C-1 Protocol spike (starts: P2, alongside the observer API)
- [ ] C-2 S1/S2 folio pages in the inspector (starts: P2-18)
- [ ] C-3 Pass timeline + fusion-group view from timing JSON (starts: P2-13)
- [ ] C-4 Flame views from profiler export (starts: P0-11 data available)
- [ ] C-5 S3 pages, provenance navigation, remarks rendering (starts: P3-13)
- [ ] C-6 Fact browser incl. reactivity-lattice overlay (starts: P4-1)
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
  - _S3 stage 2026-09-22:_ `Impeto.Checker` re-checks the TS-27 graph contract
    (unique ids, root, region resolution, span nesting, finite parent chains,
    effect scoping, edge resolution, scheduled order) over all 60 committed S3
    graphs. The checker is total and proved exact against its declarative
    specification (`violations_nil_iff`), and 7 damages each yield exactly their
    codes. It found the hand-written `dynamic-button` fixture violating span
    nesting, which the Rust validator never saw; the fixture is fixed. S1/S2
    stages remain.
- [ ] C-24 Theorem backlog: lattice laws → effect-grouping edge preservation → IVM linearity (starts: P3-4)
  - _2026-09-21:_ all three landed with P3-15; later S3 semantics extend them in the same lane.

## Documentation truth (charter #45 pending)

- [ ] C-25 Architecture-doc truth passes at each phase exit (docs claim only what ships)
- [ ] C-26 Charter/open-questions hygiene: decided items become stubs; the consumption/rule-parity matrices stay current via their staleness checks
