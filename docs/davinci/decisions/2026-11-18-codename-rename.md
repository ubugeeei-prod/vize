# Codename rename (#6832)

Status: **ready for the merge window.** This lands the 2026-11-03 decision ("Davinci is removed
everywhere except `docs/davinci/` by one scripted, replayable rename") and the three classes
decided on 2026-11-09 ([#6832](https://github.com/ubugeeei-prod/vize/issues/6832)). It replaces
the 2026-11-09 dry run, which was never merged.

## Decisions (maintainer, 2026-11-19)

- **Hard rename, no deprecated aliases.** Every public name listed under "What changes for
  users" changes in one release: no cargo feature, CLI flag, environment variable, file
  extension, counter, remark, dump header or playground id keeps its old spelling. The PR lists
  each under "Breaking changes" so the release notes carry them.
- **The refined gate measure is accepted** (see "Measurement"): `levels.codenames` counts what
  the rename script would still change, not every substring.

## Replay

`tools/support/levels/rename-codenames.ts` is the whole rename. On any branch:

```sh
node tools/support/levels/rename-codenames.ts --move      # git renames only; commit
node tools/support/levels/rename-codenames.ts --rewrite   # text, formatting, ledgers, Cargo.lock
node tools/support/levels/rename-codenames.ts --check     # lists what still carries a codename
```

Both steps print what they changed; a second run changes nothing. A branch that conflicts with
the rename takes main's side and re-runs the script.

## Mapping

| Codename | Becomes                                                                                    |
| -------- | ------------------------------------------------------------------------------------------ |
| Davinci  | `levels` as a whole lowercase word or path (`davinci/` → `levels/`), `level` when embedded |
|          | (`davinci_harness` → `level_harness`, `VIZE_DAVINCI_*` → `VIZE_LEVEL_*`); in prose `level` |
|          | as a modifier, and a plan item (`Davinci P1-7`) names the plan                             |
| Sinopia  | `l1` / `L1`                                                                                |
| Ricalco  | `l1_to_l2` / `l1-to-l2` / `L1ToL2`                                                         |
| Disegno  | `l2` / `L2`                                                                                |
| Impeto   | `l3` / `L3`                                                                                |
| Stesura  | `l4` / `L4`                                                                                |
| Folio    | `dump` / `Dump` (`.folio` fixtures → `.dump`, `FolioMode` → `DumpMode`)                    |
| Spolvero | `stages` as a whole word, `stage` when embedded                                            |

Fixed words: `davinci-differential` and `davinci-dom-differential` name the `legacy-*` features
they aliased; `vize_davinci` and `vize_davinci_derive` name `vize_l0` and `vize_l0_derive`.
A level codename counts only where a word part starts (`historicalComplete`, `portfolio` are
not codenames).

## Kept

- `docs/davinci/` is history. Its prose, names and file names stay, except references to a
  tracked path that moved, which follow it so links resolve. Plan data that tools read against
  current code (inventories, budgets, allowlists, schemas) follows fully; fix histories and
  `-records/` keep their bytes. A bare `davinci/` there names the old directory and stays.
- Captured run output (`*.log`, `*.jsonl`, `*.stderr`, `*.stdout`, `fix-history/`, `capture-*/`,
  `original-*`, `observed-*.json`, `*.observation.json`) keeps its bytes, except captures a live
  test parses (observer logs, L3 reference graphs, the dump CLI's expected stderr).
- Lines that say "codename" keep level codenames: the old L3 package and op names in the
  dependency and protocol tests, and the published old package names.
- Links into `docs/davinci/` and GitHub links pinned to a revision other than `main`.

## Measurement

`levels.codenames` measures with the script's own rules: a line counts when the rename would
still change it, so references into the kept docs and pinned links do not count, and a level
codename counts at word-part starts only. The gate imports those rules from
`tools/support/levels/rename-codenames.ts`, so the script stays in the tree after the rename, and
the gate itself moves to `tools/commands/levels/completion-gates.ts`. On main before the rename
(4a0fb417ed) the old substring measure read 3157 paths and 9082 lines; after it the gate reads 0
and 0.

## What changes for users

- Cargo features deleted: `davinci-differential` (11 crates), `davinci-dom-differential`; use
  `legacy-differential` / `legacy-dom-differential`. Renamed: `davinci-production-bench` and
  `-profile` → `level-production-*` (vize_atelier_sfc), `davinci-benchmark` and `-profile` →
  `level-benchmark*` (vize_atelier_vapor, vize_l1_to_l2).
- CLI: `vize reduce --folio-contains` → `--dump-contains`; hidden `vize build
--davinci-inject-panic` → `--level-inject-panic`; a failed build writes `<stem>.repro.dump`
  (was `.repro.folio`).
- Environment: `VIZE_DAVINCI_*` → `VIZE_LEVEL_*`, `DAVINCI_*` → `LEVEL_*`,
  `UPDATE_FOLIO_FIXTURES` → `UPDATE_DUMP_FIXTURES`, `CROQUIS_FOLIO` → `CROQUIS_DUMP`.
- Serialized: profile counters and remark ids `davinci.*` → `levels.*`; dump page headers
  `[s3-*-folio]` and `[fusion-plan-folio]` → `…-dump`; playground stage kinds `disegno` /
  `impeto` → `l2` / `l3`; playground tab id `davinci` → `levels`. Dump formats may break until
  contracts GA (2026-11-09).
- Unpublished crates: `davinci_harness` → `level_harness`, `davinci_test_support` →
  `level_test_support`.

## Removed with the rename

The deprecated feature aliases and their guard tool and test, and the six codename replay
scripts it supersedes (`rename-l0-dump-tests`, `rename-l1-surface-corpus`,
`rename-l2-span-support`, `rename-l3-dump-test`, `rename-l3-vapor-counters`,
`rename-stage-feed`) with their tests.
