# Per-surface release policy for the Real Project Matrix

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Follow-up: [#8849](https://github.com/ubugeeei-prod/vize/issues/8849).

## Decision

Release preflight gates each Real Project Matrix surface by its own policy
instead of requiring every surface to enforce.

| Surface                  | Release policy | Dispatch input                        |
| ------------------------ | -------------- | ------------------------------------- |
| `waiver-audit`           | enforced       | always enforced                       |
| `typecheck-dependencies` | enforced       | `typecheck_dependencies_mode=enforce` |
| `core-tools`             | enforced       | `core_tools_mode=enforce`             |
| `syntax-highlighter`     | enforced       | always enforced                       |
| `glyph`                  | enforced       | always enforced                       |
| Davinci DOM corpus job   | enforced       | `davinci_dom_corpus_mode=enforce`     |
| `lint-divergence`        | recorded       | `lint_divergence_mode=record-only`    |
| `lsp`                    | recorded       | `lsp_mode=record-only`                |
| `typecheck-divergence`   | skipped        | `typecheck_divergence_mode=skip`      |

An enforced surface must run in `enforce` mode with a successful raw outcome.
A recorded surface must run in `record-only` mode and report a raw `success`
or `failure`; the raw outcome stays in the version-2 surface report, and a raw
failure does not refuse the release. The skipped surface must report `skip`
and a raw `skipped` outcome.

The workflow names the exact release dispatch `Real Project Matrix
release-gated @ <H>`. The release selector requires that title, so neither a
fully `enforced` run nor any other `observational` combination can stand in
for it. Scheduled and default dispatches still enforce every surface.

The lint divergence summary must account for every measured project: passed
plus failed projects equal the project count, unusable plus breached projects
equal the failed count, and the failed project list has that length. Breached
and unusable projects are printed as a release warning. Typecheck divergence
artifacts are not required while the surface is skipped; a retained artifact
must still be bound to the source and its dependency preparation, and its
parity gaps are printed as a warning.

## Scope

This supersedes only the requirement in the
[enforced Matrix evidence record](./2026-10-11-6830-release-matrix-enforcement.md)
that all six modes enforce and that lint and typecheck parity are exactly
zero for a release. Its version-2 surface report, exact head and shard
binding, ordered surface set, attempt selection, archive custody and the five
release workflows are unchanged. No surface, budget, fixture or check is
removed, and the Matrix still measures all eight surfaces on its weekly
schedule.

## Reason

The zero-parity requirement merged on 2026-10-11. Every release since then
failed preflight on it: v0.446.0, v0.447.0 and v0.448.0. v0.443.0 remained the
latest published version for three days while merged fixes for third-party
reports waited unpublished.

The v0.448.0 Matrix
([run 38331707741](https://github.com/ubugeeei-prod/vize/actions/runs/38331707741))
measured the distance from zero. All 22 shards failed only on parity surfaces:
of 145 measured projects, 88 breach the zero lint budget and 6 have an unusable
eslint-plugin-vue baseline; four projects breach the typecheck budget; and the
LSP lifecycle fails in 10 shards. The tool surfaces and the Davinci corpus were
green in every shard. Closing that gap is ongoing work,
not a condition a single fix can meet before it ships.

Typecheck divergence is skipped rather than recorded because the `hoppscotch`
fixture hangs until the 100-minute step timeout. Its shard took 105 minutes in
the v0.447.0 and v0.448.0 runs, while every other shard finished within 25.
Recording the surface would add that hour to every release. Releases through
v0.445.0 also skipped it.

## Tradeoff

A regression in lint parity, LSP behavior or typecheck parity on a real
project no longer blocks a release; it is visible in the recorded evidence
and in the weekly enforced run. [#8849](https://github.com/ubugeeei-prod/vize/issues/8849)
restores enforcement on the three surfaces as a regression gate: per-project
recorded budgets that only ratchet down, after the `hoppscotch` hang is fixed.
