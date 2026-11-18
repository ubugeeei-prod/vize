# Real-project lint lane handoff (2026-11-18, ~02:10Z)

Do not merge this branch: it only carries the handoff.

Scope: Real Project Matrix lint divergence (#12336) plus the reka-ui stale typecheck ledger row (done in #13291). The typecheck and LSP failures belong to a separate lane.

## Where things stand

- Merged: #13291 (reka ledger row), #13286 (isolated fixtures), #13312 (bundle of five location-only Patina fixes: prefer-static-class, next-tick-style, emit nodes, no-restricted-v-bind, BOM columns).
- Open: #13278 (ssr globals prop evidence), #13287 (eslint-disable mapped aliases), #13293 (eslint-plugin-vue report nodes). The coordinator dequeued all three at 00:28Z, and a feeder script re-enqueues them. Do NOT re-arm them by hand.
- Coordinator rules in force:
  - queue-drain mode: open no new PRs until the merge queue is short and the coordinator says so;
  - cap of 6 open PRs;
  - one workflow_dispatch in flight at a time;
  - bundle the tooling-only branches;
  - keep each Patina behaviour change as its own PR (location-only Patina fixes for different rules may share one PR).

## Local measurement (integration of every lane branch, 59 hydrated projects)

Compared with run 44093557816, on the 57 projects both runs share:

| | FP | FN |
| --- | --- | --- |
| run 44093557816 | 5230 | 3645 |
| integration 1 (21 branches) | 69 | 10 |
| integration 2 | 30 | 2 |
| integration 3 (all 27 branches, final) | **5** | **2** |

Over all 59 projects, integration 3 leaves 31 FP and 6 FN:

- Davinci HTML tree construction:
  - shadcn-vue nested `<form>`: 26 parser FP, plus 3 v-on-handler-style FN on the attributes of the dropped element;
  - habitica `<p><ul>…</ul></p>`: invalid end tag, 1 FP;
  - vuestic-ui VaDataTable foster parenting: 1 FP.
- Davinci duplicate-attribute branch (`fix/patina-duplicate-attribute-error`, handed off): the vue2-elm FP/FN pair and the nativescript-vue GH1010 FN.
- Parser: mint-ui ESC control character (`control-character-in-input-stream`), 1 FN.
- Ours, the custody re-pin: gui-for-singbox (bare `#`) and mobile-web-best-practice (class-decorator prop), 1 FP each.
  - `tests/_fixtures/differential/linter/reference-coverage-8849/contract.json` pins eslint-plugin-vue 10.9.2, but the all-mapped matrix baseline runs the 10.12.0 scorecard runtime. The cases therefore never match, and `reviewedReferenceFindings` keeps them as FPs, by design.
  - The coordinator assigned this to the lane: after the lane PRs land, re-review both cases under 10.12.0 using packets from a CI all-mapped run (each project then has exactly that one FP), add the runtime or cases to the contract, and update its pinned `cases.length === 2`.

The branch `wip/real-project-lint-integration` (490db0a419) is the exact merge used for integration 3. To reproduce:

```sh
cargo build -p vize --profile ci --bin vize
.handoff/tools/local-lint.sh <out> . <comma-separated project ids>
node .handoff/tools/sum-runs.mjs <baseline-artifacts> <out>
```

## Branches, grouping and order

Every branch is on origin. "Next" says what to do when the coordinator allows new PRs. PR bodies are in `.handoff/pr-bodies/`.

### Tooling bundles (open first, harness then classifications)

| Branch | Head | PR body | Next |
| --- | --- | --- | --- |
| `fix/lint-divergence-harness` | 55903363c6 | `pr-harness.md` | Open (rebased on main after #13286). It bundles baseline-script-locations, upstream-default-options (v-on-handler-style matrix options) and the Vue 2 dialect. Its `main.ts` already combines isolation, matrix config and dialect. |
| `fix/lint-divergence-classifications` | fd7c545e2e | `pr-classifications.md` | Open after the harness PR. It bundles documented-extensions plus the imported-contract classifiers (squashed into one commit). |

Their component branches are superseded by the bundles; keep them only for reference: `fix/lint-baseline-script-locations` (1f013b67cd), `fix/lint-divergence-upstream-default-options` (6f60c55270), `fix/lint-divergence-vue2-dialect` (d75871080c), `fix/lint-divergence-documented-extensions` (f44c08c1a1), `fix/lint-divergence-imported-prop-contracts` (2b177ee697).

### Patina behaviour changes (one PR each, in this order)

| # | Branch | Head | PR body | Note |
| --- | --- | --- | --- | --- |
| 1 | `fix/patina-no-props-shadow-sibling-script` | 1a4bafeae5 | `pr-propsshadow.md` | The heritage-only shadow FPs it surfaces are classified in the classifications bundle, so open that bundle first. |
| 2 | `fix/patina-unused-emits-escapes` | 3b1d5ef099 | `pr-unused-emits.md` | Approved: a computed `emit(x)` means no event can be proven unused. Includes its own tooling classifiers (instance escape, computed and resolved names). |
| 3 | `fix/patina-decorator-components-registration` | 652004dbd4 | `pr-decorator.md` | Includes the gridea casing classification. |
| 4 | `fix/patina-no-duplicate-class-upstream-nodes` | 6401b5e2f3 | `pr-ndc.md` | Battery census +1/+1/+20. |
| 5 | `fix/patina-no-multi-spaces-block-tags` | cea3d12937 | `pr-nms.md` | |
| 6 | `fix/patina-nuxtjs-composition-api` | 40cf391f68 | `pr-nuxtjs.md` | |
| 7 | `fix/patina-no-empty-pattern-slot-scope` | b5ef38de52 | `pr-emptypattern.md` | `compatibility_template/tests.rs` is near the 500-line cap. |
| 8 | `fix/croquis-trailing-operator-member` | c66096eef4 | `pr-croquis-operand.md` | A croquis change: watch the `croquis_analyze_full_large` instruction ceiling in the merge group. |
| 9 | `fix/patina-self-name-last-extension` | 7b9ee94672 | `pr-selfname.md` | |
| 10 | `fix/patina-no-dupe-keys-model-binding-once` | 6e4cf25318 | `pr-dupekeys-model.md` | |
| 11 | `fix/patina-valid-default-unknown-reference` | 093e5d1b6c | `pr-valid-default.md` | Coordinator decision (b) A. |
| 12 | `fix/patina-content-outside-block-text-only` | 8a3229c4a2 | `pr-content-text.md` | Coordinator decision (a) A. A separate rule idea, `vue/no-dropped-top-level-element`, is noted in the body. |
| 13 | `fix/patina-template-expression-disable-comments` | a50faf1f18 | `pr-expr-comments.md` | Stacked on #13287; rebase after #13287 merges. |

Superseded by #13312 (merged), so they can be deleted: `fix/patina-prefer-static-class-upstream-nodes`, `fix/patina-next-tick-style-identifier`, `fix/patina-emit-report-nodes`, `fix/patina-no-restricted-v-bind-key`, `fix/patina-output-bom-columns`, `fix/patina-upstream-report-locations`.

Handed to Davinci: `fix/patina-duplicate-attribute-error` (no PR).

## Before opening each PR

- Rebase onto origin/main.
- `tests/_fixtures/differential/linter/manifest.json` conflicts: take main's file, then run `node .handoff/tools/merge-manifest.mjs <worktree> <branch>…`, which re-adds each branch's new cases. Alternatively, re-add one case with `.handoff/tools/add-linter-case.ts <wt> <id> <root> file:role…`.
- `tools/support/fixtures/lint_divergence/records.ts` conflicts between documented-extensions and imported contracts: `node .handoff/tools/resolve-records.mjs <file>`, which handles the 4-hunk shape. git rerere is enabled in the worktrees.
- `crates/vize_patina/src/output/shared.rs` is a registered report source: append its new sha256 to `CURRENT_REPORT_SOURCES` in `tests/differential/linter-api.ts` (as #13312 did).
- Run:
  - `cargo test -p vize_patina` (wrapped in the cargo slot);
  - `cargo clippy -p vize_patina --lib`;
  - `node docs/scripts/generate-patina-rules-page.ts` and commit any source-anchor changes;
  - `tests/tooling/differential-linter-api.test.ts`;
  - `patina-shared-parse-gate.test.ts`;
  - the `lint-*` tooling tests. In this environment the CLI-binary tests fail the same way on main.
- `restrict_plus_generic_intersection` in vize_patina is flaky under load and passes alone.

## Coordinator decisions to remember

- Standing rule: when Patina is verified correct and the difference comes from a limitation of the reference, classify it narrowly with a specific reason and tests for both sides, without asking. Ask only when (a) Patina may be wrong and fixing it changes user-visible default behaviour, (b) a default or preset changes, or (c) a classification is broad enough to hide a same-file case.
- Dynamic emit names: yes (done in `fix/patina-unused-emits-escapes`). The type-informed refinement for parameters annotated with string-literal unions is not done, because this rule has no expression types. Syntactic resolution covers const and `as const` maps.
- The dropped "computed emit name" classifier must not come back as written: it turned shared findings into FPs. The current classifier mirrors Patina's own resolution.
- Never reference private repositories in public text. Never dispatch release.yml. No `git stash`. No Python. No new PRs while the queue is draining.

## Tools (`.handoff/tools/`)

- `local-lint.sh`: local all-mapped lint divergence over projects. Set WT to your worktree.
- `sum-runs.mjs`: FP/FN deltas between two artifact trees.
- `check-classify.ts`, `check-pairs.ts`, `diff-runs.mjs`: per-rule triage helpers.
- `add-linter-case.ts`, `merge-manifest.mjs`: differential manifest cases.
- `resolve-records.mjs`, `apply-matrix-main.mjs`: conflict recipes for records.ts and main.ts.
- `pr-event.sh`: exits on the first merge, close, dequeue or failing check of the listed PRs.
- `cargo-env.sh`: the Rust 1.100.0 toolchain env used here. Adjust its paths on a new machine.
