# Level dump naming (#6832)

Collapses the #6906..#6947 stack into one change. Level crates have no
compatibility users, so no aliases or fallback readers are added.

- **L3 package identity.** `crates/vize_impeto` moves to `crates/vize_l3`
  (move-only commit) and the package/dependency name becomes `vize_l3`.
  First-name semver checks use the published `vize_impeto@0.429.0` baseline.
- **Stage view paths and feed types.** Playground `features/davinci/` becomes
  `features/stages/`; DOM witnesses drop the `davinci_` prefix. The feed family
  becomes `StageFeed`, `StagePage`, `StageRemark`, `StageNegotiation`; its JSON
  schema and negotiation behavior are unchanged.
- **Conversion registry.** `L2_TO_L3` is registered (metadata only).
- **Dump API.** Shared `vize_davinci::dump::{Dump, Mode, Error}`, the `Dump`
  derive with `#[dump(name = ...)]`, and per-level `dump::Page` replace the
  codename page types.
- **Versioned dump protocols.** L2 writes `[l2-dump-v2]` (published extension
  `s2-page@1` keeps its historical grammar via `dump::historical::v1`). L3
  writes `[l3-dump-v2]` with sixteen `l3.*` op mnemonics. The codec is chosen by
  typed constructor; there is no header inference.
- **Hash domains.** Artifact-key, key-manifest, SFC and global summary domains
  move to `.v2`; old cache records miss. Compiler output is unchanged.
- **CLI.** `vize dump --level l1|l2|l3 --roundtrip FILE` checks byte identity of
  a parse/print roundtrip. Exit 0 identity, 1 failure, 2 usage.

Instruction ceilings: the renames moved the process memory map, and
`stacker`'s first-use stack probe parses `/proc/self/maps` inside the measured
window. The harness now takes that probe before any window (the v-for lowering
probe drops from ~150k to ~21k instructions). The patina markup visit also
reads its rule name once and pre-sizes the content-model fact maps.

Dropped from the stack: one-shot rename/replay scripts under
`tools/support/levels/` and their tests, and the per-slice evidence records.
Legacy products stay byte-exact. The proposed formal/Lean namespace move was
closed in favor of Kani on production Rust (see the central decision record).
Remaining #6832 work is shared production capture for CLI and playground.

## One native stage capture

The stage ladder now collects pages, optimization remarks, steps and walks in
one L1 → L2 → L3 execution. Its L2 remark observer shares the executed pass
plan with the page collector; `analyzeSfc` consumes those remarks instead of
running a second L2 transform solely for the feed. This keeps the existing
feed schema and the inline HTML remark filter. The CLI export below is a
consumer of this capture. Neither change establishes that legacy product
compilation runs through the levels.

## All-level CLI export

The description below records the earlier ladder CLI. It was superseded by
the same-run product CLI in [production stage capture](./2026-09-28-production-stage-capture.md):
the current command accepts `.vue` and Pug inputs and `--pipeline` selects a
real DOM, SSR or Vapor backend. The old ladder is not product evidence.

`vize dump --all-levels --json FILE` runs the same native `ladder_run` that
feeds the playground stage view and writes its pages and remarks through the
single `StageFeed::to_json` serializer. The input is a raw Vue template file;
`.vue` SFC input fails explicitly while the native L1 container remains open
in #7039; Pug input also fails until its native stage adapter is wired. The
browser view's feed uses the same template bytes after its own
SFC extraction. The existing level roundtrip command keeps its success and
failure bytes. This feed covers the current native L1 → L2 → L3 ladder,
separate from the current legacy-backed product compiler path.

At the time of this earlier decision, `--pipeline` migration and removal of the `davinci-opt` bin remained open. The
old binary still tests a no-op pass catalogue over input dumps; it does not
serve as evidence that product compilation uses the new levels.

## Differential feature selectors

The current differential test selectors `davinci-differential` and
`davinci-dom-differential` become `legacy-differential` and
`legacy-dom-differential`. Their Cargo dependency values, cfg gates, explicit
Actions recipes, corpus helpers, current suite commands and tooling assertions
move together. This is a feature-name change only: corpus environment variables
and emitted messages, test target names, profile keys, archived fixtures and
compiler output stay as they are. The scoped
`tools/support/levels/rename-differential-features.py` script replays the
change on the then-current main and skips fixture/snapshot paths.
