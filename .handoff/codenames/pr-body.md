Removes the Davinci codenames (Davinci, Sinopia, Ricalco, Disegno, Impeto, Stesura, Folio, Spolvero) from every path, identifier and string outside `docs/davinci/`, closing `levels.codenames` (#6832). The whole change is one scripted, replayable rename: `tools/support/levels/rename-codenames.ts`.

- `levels.codenames`: **0 paths / 0 lines** (main: 3157 paths / 9082 lines on the old substring measure).
- 3191 moves (`--move`, move-only commit), ~1200 rewritten files (`--rewrite`), then a short series of hand fixes, each its own commit.
- Recorded in `docs/davinci/decisions/2026-11-18-codename-rename.md` (decisions of 2026-11-19: hard rename without aliases; the refined gate measure).

## Breaking changes

There are no deprecated aliases. Every old spelling stops working in this release.

**Cargo features**
- `davinci-differential` (11 crates) and `davinci-dom-differential` (`vize_atelier_sfc`) are deleted. Use `legacy-differential` / `legacy-dom-differential`.
- `davinci-production-bench` / `davinci-production-profile` → `level-production-bench` / `level-production-profile` (`vize_atelier_sfc`).
- `davinci-benchmark` / `davinci-benchmark-profile` → `level-benchmark` / `level-benchmark-profile` (`vize_atelier_vapor`, `vize_l1_to_l2`).

**Rust API**
- `vize_atelier_vapor::compile` options: field `davinci_retained_lane` → `level_retained_lane`.
- `vize_incremental` artifact: field `folio` → `dump`.
- `vize_carton::timing_observer::WALK_KEY`: `"davinci.pass.walk"` → `"levels.pass.walk"`.

**CLI**
- `vize reduce --folio-contains` → `--dump-contains`.
- Hidden `vize build --davinci-inject-panic` → `--level-inject-panic`.
- A failed build writes `<stem>.repro.dump` (was `.repro.folio`); `vize repro` and `vize reduce` read `repro.dump`.

**Environment variables**
- `VIZE_DAVINCI_*` → `VIZE_LEVEL_*` (`VIZE_LEVEL_DIFFERENTIAL_CORPUS`, `_SHARD`, `_SHARD_VECTOR`, `VIZE_LEVEL_FACT_CORPUS`, `_WITNESS`, `VIZE_LEVEL_METAMORPHIC_CORPUS`, `VIZE_LEVEL_COMPLEXITY_CORPUS`, `VIZE_LEVEL_RENDERED_CORPUS`, `VIZE_LEVEL_REACH_REFUSAL_SAMPLES`, `VIZE_LEVEL_DOM_CORPUS_MODE`, `VIZE_LEVEL_DOM_CORPUS_OUTCOME`, `VIZE_LEVEL_UNSUPPORTED_CENSUS_CORPUS`).
- `DAVINCI_*` → `LEVEL_*` (`LEVEL_BASELINE_REFRESH`, `LEVEL_RENDER_SNAPSHOTS`, `LEVEL_HARNESS_GLOBAL_ALLOC`).
- `UPDATE_FOLIO_FIXTURES` → `UPDATE_DUMP_FIXTURES`; `CROQUIS_FOLIO` → `CROQUIS_DUMP`.

**Serialized names**
- Profile counters and remark ids `davinci.*` → `levels.*`.
- Dump page headers `[s3-*-folio]` / `[fusion-plan-folio]` → `…-dump`. Historical v1 L2 pages (`[disegno]`) still decode.
- Playground stage kinds `disegno` / `impeto` → `l2` / `l3`; playground tab id `davinci` → `levels`.

**Repository layout (contributors)**
- `davinci/` → `levels/`; `tools/commands/davinci/` → `tools/commands/levels/`; `tools/support/compat/davinci/` → `tools/support/compat/levels/`.
- Crates `davinci_harness` → `level_harness`, `davinci_test_support` → `level_test_support`.
- Test, bench and example targets `davinci*` → `level*` / `levels`.
- Workflows `davinci-*.yml` → `level-*.yml`.

## Measurements

MEASUREMENTS

## Replay on another branch

```sh
node tools/support/levels/rename-codenames.ts --move      # commit (move-only)
node tools/support/levels/rename-codenames.ts --rewrite   # commit
node tools/support/levels/rename-codenames.ts --check     # prints "0 paths still named"
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)
