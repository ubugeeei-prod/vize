# Codename rename (#6832) — draft

Status: **replay ready; two questions open** (below). The rename lands as one scripted PR in a
quiet merge window that the Davinci orchestrator picks (maintainer decision 2026-11-09). The
program name was decided on 2026-11-03; three remaining classes were decided on 2026-11-09
([#6832 comment](https://github.com/ubugeeei-prod/vize/issues/6832#issuecomment-6734570272)). This
page records how the level codenames go and lists every public or serialized name that changes.

## Replay

The branch is a replay, not a hand-edited snapshot. On the window's base:

1. Cherry-pick the two script commits (`tools/support/levels/rename-codenames.ts`, the gate and
   their tests) and the three hand-edit commits: name-origin prose and playground display names;
   deleting the deprecated program-named feature aliases; alias lines for the old L3 package and
   op names.
2. `node tools/support/levels/rename-codenames.ts --move`, then commit (move-only).
3. `node tools/support/levels/rename-codenames.ts --rewrite`, then commit, including the new
   ledger files. This reformats the touched files and regenerates the path-derived ledgers (v-on
   corpus, croquis consumption, SourceLocation inventory, consumer migration surfaces).
4. Cherry-pick the four post-rewrite fixes: alias comments that a formatter moved off their line;
   the L2 v1 wire tests; the re-recorded digests of rewritten files (host-path and storage-bridge
   fixtures, warm type-backed manifests, observer log captures, differential runtime packs, the
   native beta DTO custody); and the two tests that read pre-rename captures through the rename.
   If a pin no longer applies, re-record it from the rewritten file rather than editing it by hand.
5. `--check` should report 0, and `node tools/commands/levels/completion-gates.ts` should show
   `levels.codenames 0 paths, 0 lines`.

A PR that conflicts with the rename follows it by re-running the script.

## Mapping

| Codename | Becomes                                                         | Examples                                                                                                        |
| -------- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Davinci  | `levels` as a whole word, `level` when embedded (decided 11-03) | the davinci directory → `levels/`, davinci_test_support → `level_test_support`, VIZE_DAVINCI_* → `VIZE_LEVEL_*` |
| Sinopia  | `l1` / `L1`                                                     | old package name `vize_sinopia` (kept, see below)                                                               |
| Ricalco  | `l1_to_l2` / `l1-to-l2` / `L1ToL2`                              | old package name `vize_ricalco` (kept)                                                                          |
| Disegno  | `l2` / `L2`                                                     | `DisegnoFolio` → `L2Dump`                                                                                       |
| Impeto   | `l3` / `L3`                                                     | the impeto-ops page → `l3-ops`, `ImpetoFolio` → `L3Dump`                                                        |
| Stesura  | `l4` / `L4`                                                     | crate-doc prose                                                                                                 |
| Folio    | `dump` / `Dump`                                                 | folio fixture files → `.dump`, `FolioMode` → `DumpMode`, the folio_laws test → `dump_laws`                      |
| Spolvero | `stages` as a whole word, `stage` when embedded                 | `SpolveroFeed` → `StageFeed`                                                                                    |

The Folio, Spolvero and Impeto rows follow the mapping proposed in the #6832 body.

## Rules

- Case and separators are preserved: `CROQUIS_FOLIO` → `CROQUIS_DUMP`, `s3-values-folio` →
  `s3-values-dump`.
- A level codename counts only where a word part starts. `historicalComplete` (`…ricalco…`) and
  `portfolio` (`…folio`) are not codenames. The `levels.codenames` gate now measures the same way;
  until now these accounted for most of its Ricalco count.
- In Markdown under `docs/`, level codenames stay. They are allowed aliases there, and most
  mentions name historical things: published packages, old wire headers. Only file paths follow
  their moves, including a codename used as a file extension (`x-baseline.folio`). Prose in
  `docs/davinci/` keeps the program name, as decided on 11-03.
- Other data under `docs/` (schemas, TOML, TSV, JSON) names current code and follows the rename.
- A line that says "codename" keeps its level codenames (the gate's alias form). This keeps the
  remaining identities below.
- Captured run output keeps its bytes: `*.log`, `*.jsonl`, `*.stderr`, `*.stdout`, `fix-history/`,
  `capture-*/`, `original-*`, `observed-*.json`, `*.observation.json`. So do pinned GitHub links
  and links that resolve into `docs/davinci/`.
- Captures a live test parses are test inputs and follow the rename: the canonical observer logs,
  the generated L3 reference graphs and the dump CLI's expected stderr.
- Records that pin git trees of past revisions keep those revisions' paths (the css-hover
  custody record); their tests compare them through `renamePath`. A digest-pinned captured
  GitHub job keeps its bytes; its test matches step names through `rewriteText`.

## Kept identities (alias lines)

- The L2 v1 page header `[disegno]` / `[disegno.ops]` (`HistoricalV1` grammar), so saved v1 pages
  still parse. Its wire tests spell it once through an alias constant.
- The historical L3 op name `impeto.set-text`, used in the test that rejects old vocabularies.
- Old published package names `vize_sinopia`, `vize_disegno` and `vize_ricalco`, in the
  stage-dependency guards and the consumer-migration compatibility lists.

## What breaks

Published or user-visible:

- Cargo features deleted: davinci-differential (12 crates) and davinci-dom-differential, the
  deprecated aliases of `legacy-differential` and `legacy-dom-differential`; enable the
  `legacy-*` names instead. A release that ships this needs a minor version bump for
  cargo-semver-checks.
- Cargo features renamed: davinci-production-bench and -profile → `level-production-bench` and
  `level-production-profile` (vize_atelier_sfc); davinci-benchmark and -profile →
  `level-benchmark` and `level-benchmark-profile` (vize_atelier_vapor, vize_l1_to_l2).
- Environment variables: VIZE_DAVINCI_* → `VIZE_LEVEL_*` (`…_DIFFERENTIAL_CORPUS`, `…_DOM`,
  `…_FACT_CORPUS`, `…_METAMORPHIC_CORPUS`, …). `UPDATE_FOLIO_FIXTURES` → `UPDATE_DUMP_FIXTURES`,
  `CROQUIS_FOLIO` → `CROQUIS_DUMP`, UPDATE_DAVINCI_ROUTES → `UPDATE_LEVEL_ROUTES`.
- CLI flags: --davinci-inject-panic → `--level-inject-panic`, `--folio-contains` →
  `--dump-contains`.
- Rust names in workspace crates: `trait Folio` → `Dump`, `FolioValue` → `DumpValue`, every
  `mod folio` → `mod dump`, `check_folios` → `check_dumps`, test crates davinci_* → `level_*`.
  davinci_harness and davinci_test_support (unpublished) → `level_harness`,
  `level_test_support`.

Serialized (saved data with the old spelling stops parsing):

- Dump page headers: `[s3-values-folio]`, `[s3-placement-folio]`, `[s3-extraction-folio]`,
  `[s3-partition-folio]`, `[s3-reactivity-folio]`, `[fusion-plan-folio]` (and their `.section`
  headers) → `…-dump`. The `#[dump(name = …)]` derives change with them.
- Counter and remark ids davinci.* → `levels.*` (davinci.expr._, davinci.rank._, …).
- Playground stage-page kinds `disegno` / `impeto` → `l2` / `l3`. The rung display names become
  **Semantic** and **Reactivity**.
- `.folio` fixture files → `.dump`.

Tooling: the `level-differential-feature-rename` tool and test that kept the deprecated aliases
published are deleted with them.

## Decided (2026-11-09)

1. Deprecated program-named feature aliases are deleted, not renamed.
2. Captured logs that live parsers read in tests are rewritten to the new spelling; archives
   nothing parses keep their bytes.
3. Non-Markdown data under `docs/davinci/` that names current code follows the rename; Markdown
   prose keeps level codenames as aliases.

## Open questions for the maintainer

1. The s3 page headers break saved dumps. Accept the break, or mint `l3-*-dump-v2` headers with
   v1 parse aliases, as L2 did with `[l2-dump-v2]`?
2. Keep L2 v1 (`[disegno]`) parsing, as drafted, or drop it with this rename?

## Not verified locally

The replay passes `cargo check --workspace --all-targets`, `cargo fmt --check`, the gate, the
touched tooling tests and the Rust tests that read rewritten captures (dump CLI, L3 reference
and lattice fixtures, L2/L3 dump protocol). The other Rust test suites run in CI only. 118 tracked files hold the
pre-rewrite digest of a rewritten file; most are historical receipts, and any that a Rust test
checks against the current file needs its pin re-recorded in step 4.
