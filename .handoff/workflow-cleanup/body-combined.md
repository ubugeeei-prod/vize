## Summary

One PR for the workflow cleanup's guard and deletion steps, combined at the lead's request: separate PRs kept conflicting with each other and each push started ~35 workflow runs. It replaces #13605, #13610, #13606, #13609, #13607, #13608 and #13626. Their full descriptions are below, one per section.

- **Workflows: 260 → 125** on main (280 before the cleanup started; #13604 already removed 20). All of them are CI evidence harnesses for single fixes, dead dispatch-only studies, or feature-gated test carriers whose tests now run in the new Rust feature lane.
- **Guard:** workflow files are `NN-<subject>.yml` from now on. Protected release, trusted-publishing and Pages files keep their names. The generator rejects stale driver, script or part directories.
- **Feature lane:** 23 workflows that were the only CI for experimental Cargo features become rows of one manifest. Affected rows run in the PR Rust products tier, and all 33 rows run in full qualification (`01-rust-feature-tests.yml`). `--ignored` measurement tests stay in the code but the lane doesn't run them.
- **Kept workflows lose their one-off modes:** content-mapper conformance, criterion, release contracts, CI wall time, source comparison, Vite+ config and n8n.
- **Tools code:** the scripts and tests that served only the deleted workflows are deleted. Each file was checked with `git grep` for surviving referrers.
- **Not deleted, with reasons:** two workflows (`typechecker-native-phases`, `davinci-canon-scaling`) stay because custody tests read their bytes. `language-project-fixtures` stays for its custody chain. The proof-only jobs in four workflows move to the proofs workflow later.
- **Byte pins:** refreshed for the edited kept workflows (`content-mapper-conformance`, `release-contracts`). The entries of deleted workflows are removed, along with their archives.

## Verification (local, on this branch)

- Generator `repository-config.ts --check` (1011 outputs), `v1-support-matrix.ts --check`, `verify-tool-layout.ts`, `vp check`, and `source-file-lengths.ts` against the merge base all pass.
- Assertion lint: `rust-script tools/commands/davinci/assertion-lint.rs` reports OK.
- `node --test` over 126 tooling files: every edited test, `workflow-layout`, `github-workflows*`, `release/*`, `automation-commands`, `rust-cache-docs-*`, `retired-tooling-languages` and `rust-feature-tests*`. 746 pass and 13 fail. The same 13 fail on main in this environment (GNU `sha256sum`/`tar` in `setup-wild-http-recovery`, live registry reads).
- Feature lane evidence: run 45329598659 ran all 33 rows and all passed. Links per row are in the H section.

Known red check, not caused by this PR: `canon-upstream-diagnostics.test.ts` › `slots` (TS2345 on `main.vue:23:11`) also fails on main-merged PR runs after #13603. A separate fix is in progress, and this test is neither skipped nor weakened.

## Product impact

CI and tooling only. No crate, `davinci/` or npm package source changes, so the product binaries are unchanged.

<details><summary>#13605 — ci: add the workflow naming and layout guard</summary>

#### Summary

First PR of the workflow cleanup. It adds the guard. No workflow changes yet.

- **Naming rule.** Workflow files are `NN-<subject>.yml`, where `NN` is the catalog group (01–11). `tools/support/ci/workflow-names.ts` rejects:
  - unprefixed names;
  - investigation words in the subject (`public`, `published`, `pair`, `paired`, `replay`, `original`, `custody`, `packets`, `evidence`, `candidate`, `frontier`);
  - issue or version numbers;
  - words the group already implies (`bench*` in 03, `maintenance` in 11);
  - a subject already used in another group.
- **Protected files** keep their exact names through an explicit allowlist that also assigns their group: `release.yml`, `release-jsr.yml`, `release-verify.yml`, `release-cadence.yml`, `release-npm-bootstrap.yml`, `pkg-pr-new.yml`, `published-fresh-project.yml`, `parcel-transformer.yml`, `deploy-docs.yml`. These names are bound by npm/JSR/crates.io trusted publishing, GitHub Pages OIDC, the Parcel publisher or `uses:` from a protected file.
- **Legacy allowlist.** Today's 271 other names are listed. Nothing may join the list. The renames at the end of the cleanup remove it. Deleted names may stay in it until then, so the parallel deletion PRs do not conflict on this file.
- **Catalog.** `workflow-catalog.ts` now derives the group from the prefix (or the protected allowlist) and asserts that the Pkl source sits in the matching group directory.
- **Same-stem invariant in the generator.** Every directory under `.github/scripts/workflows/`, `tools/config/pkl/scripts/.github/scripts/workflows/` and `tools/config/pkl/parts/` must belong to an existing workflow stem, apart from the shared helper directories. Today the generator does not delete stale outputs, so a deleted or renamed workflow could leave orphaned drivers behind. Now it can't.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`: 1764 outputs, 0 updated.
- `node --test tests/tooling/workflow-layout.test.ts`: 11/11, including the new rule test with negative cases.
- `node tools/commands/ci/verify-tool-layout.ts`: passes.
- A stray `tools/config/pkl/parts/ghost/` directory makes the generator fail with `no workflow has this stem`.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

<details><summary>#13610 — ci: delete dead workflows</summary>

#### Summary

Second PR of the workflow cleanup. It deletes 17 dead workflows: no caller, a hard gate to one old PR, or only failing or stale dispatch runs. Their drivers, Pkl sources and the `tools/` code and tests that served only them are deleted as well (about 240 files removed in total).

| Workflow                                            | Why it is dead                                                   | What still covers the guard                                                                                                                           |
| --------------------------------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| queue-release-gates                                 | Orphan `workflow_call` since #9198 unwired it from Check; 0 runs | Docs build, Fuzz and Miri keep their schedules and dispatch. Their now-unused `workflow_call` triggers (and the docs `links-only` input) are removed. |
| asset-import-bindings                               | `if: pull_request.number == 10711`; skipped 20/20                | —                                                                                                                                                     |
| for-bindings                                        | `if: pull_request.number == 10726`; skipped 20/20                | —                                                                                                                                                     |
| babel-rc9-frontier                                  | One failed push run (10-25)                                      | Babel plugin host contracts                                                                                                                           |
| n8n-script-rule-controls, n8n-whole-corpus-observer | One failed dispatch each (10-16)                                 | n8n adoption (daily)                                                                                                                                  |
| plain-script-import-public-acceptance               | Never ran                                                        | —                                                                                                                                                     |
| typechecker-syntax-semantic-completion              | Dispatch-only, last run 10-25                                    | —                                                                                                                                                     |
| vts-source-census                                   | Dispatch-only, last run 11-08                                    | —                                                                                                                                                     |
| doctor-filter-bench, compact-collections            | Dispatch-only measurements, last runs 10-13 to 10-20             | Criterion / source-comparison A/B dispatch                                                                                                            |
| rust-frontend-threads-bench                         | Three-run rustc parallel-frontend experiment                     | —                                                                                                                                                     |
| vuefes-ssr-style-dependencies                       | Dispatch-only, last run 10-11, 2/9 green                         | —                                                                                                                                                     |
| source-symbolizer-smoke, source-symbolizer-retained | Forensic symbolizer; skipped 20/20 on PRs                        | —                                                                                                                                                     |
| native-ops-incident                                 | Forensic incident trace with an authorization input              | —                                                                                                                                                     |
| v1-exact-candidate                                  | No caller and never ran (maintainer decision: delete)            | `tools/commands/release/v1-exact-candidate.ts` stays: the v1 matrix workflow runs it                                                                  |

Tooling deleted with them (each file checked with `git grep`: no surviving referrer outside historical records):

- `tools/support/compat/github/plan-release-gates.ts`
- the `native-phase-artifact*` helpers
- the `vuefes` oracle helpers
- the `source-symbolizer-*`, `vts-source-census*` and `rust-frontend-threads-*` scripts
- the `.github/scripts/native-ops*` strays
- their tests, along with their keys in `tooling-test-timings.json` and `retired-tooling-languages.json`

Left in place on purpose: `typechecker-native-phases.yml` (daily, failing 20/20) and `davinci-canon-scaling.yml` (dispatch-only). The `typechecker-native-*` source-recipe and `empty-vue-star-*` star-listing custody tests read their bytes, so it leaves together with that family. Also left: the `typechecker-native-*` / `empty-vue-star-*` / `javascript-sfc-*` benchmark family. Part of it is still alive through `tools/config/pkl/NativeChecks.pkl` and its own tests, so a dedicated tools sweep should untangle it.

Historical decision records and fixtures keep the old workflow names.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `node tools/commands/ci/verify-tool-layout.ts`.
- `node --test` over `workflow-layout`, `github-workflows*`, `release/*`, `docs-quality-source-scope`, `pr-performance-observation`, `rust-cache-docs-*` and `retired-tooling-languages`: 785 tests. The only failures are 2 cases in `release/verify-published-registry.test.ts`, and they fail the same way on main.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

<details><summary>#13606 — ci: delete compiler evidence workflows</summary>

Part D of the workflow cleanup: delete the group-06 (compiler) evidence workflows.

#### Deleted workflows (25)

Each one recorded evidence for one fix or one investigation: issue-numbered packets, pinned public-release baselines, before/after captures, forensic profilers. The assertions that outlive them already run in the normal suite (vize integration and crate tests, plus the tooling tests that the affected-tooling and full-tooling lanes select).

| Workflow                              | What it recorded / where the guard stays                                                                                                                                                                                                                                                            |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ambient-template-ref-ownership.yml`  | #10980 original/repair packets.                                                                                                                                                                                                                                                                     |
| `async-component-ancestry.yml`        | Pinned-parent precision controls for one change; its workflow test goes with it.                                                                                                                                                                                                                    |
| `binding-origin-laws.yml`             | Laws in native-api-wire-origin.test.ts stay in tooling.                                                                                                                                                                                                                                             |
| `compiler-options-snapshot.yml`       | Old-vs-candidate functional programs against the immutable parent.                                                                                                                                                                                                                                  |
| `cross-file-script-offset.yml`        | #10919 evidence pinned to vize@0.601.0.                                                                                                                                                                                                                                                             |
| `css-color-declaration-admission.yml` | #10562 packets; lsp-css-color-declaration-admission.test.ts + vize_maestro document_color tests stay in the suite.                                                                                                                                                                                  |
| `css-color-literal-hover.yml`         | #10556/#10562; same tests as above.                                                                                                                                                                                                                                                                 |
| `css-compile-source-maps.yml`         | Source-map ABBA evidence. `pr-js-checks` still runs `tests/native/css-compile-source-maps.mjs` and `css-v-bind-source-maps.mjs`; `css-source-map-profile.test.ts` and the vize_atelier_sfc tests stay. Its event-source resolver (`tests/native/css-source-map-event.mjs`) and its test go with it. |
| `defaulted-inject.yml`                | Before/after evidence pinned to vize@0.602.0; defaulted-inject-snapshot-transition.test.ts stays in tooling.                                                                                                                                                                                        |
| `expression-overlap-audit.yml`        | #10845 audit (nightly parser instruction attribution).                                                                                                                                                                                                                                              |
| `jsx-backend-origins.yml`             | Before/after backend overhead evidence; jsx-*-profile-contract tests stay in tooling.                                                                                                                                                                                                               |
| `narrowed-slot-host-guard.yml`        | #10660 reported-package evidence pinned to 0.569.0.                                                                                                                                                                                                                                                 |
| `native-build-token-layout.yml`       | Release-producer token layout check already covered by 02-release-smoke image-token laws and native-build-token-layout.test.ts.                                                                                                                                                                     |
| `native-checker-cpu-metadata.yml`     | CPU attribution resolution for one capture (capture_run_id input).                                                                                                                                                                                                                                  |
| `native-sfc-runtime-options.yml`      | Before-source packet evidence; native-sfc-runtime-options.test.mjs stays in tooling.                                                                                                                                                                                                                |
| `native-sfc-runtime-version.yml`      | native-sfc-runtime-version.test.mjs stays in tooling.                                                                                                                                                                                                                                               |
| `provide-type-assertion.yml`          | 27-packet before-evidence; vize/vize_croquis tests run in the suite.                                                                                                                                                                                                                                |
| `recovery-normal-codegen.yml`         | Allocator-shim correlation for one change.                                                                                                                                                                                                                                                          |
| `sfc-parse-attributes.yml`            | Source-role packet evidence; sfc-parse-attributes-*.test.ts stay in tooling.                                                                                                                                                                                                                        |
| `template-alias-paired-11822.yml`     | Issue-numbered (#11822) paired mapper comparison; the v1 alias row now cites `crates/vize/tests/check_canon_path_alias_json_cli.rs` (next to `check_default_imports_cli/aliased_diagnostics.rs`).                                                                                                   |
| `template-global-policy.yml`          | Evidence pinned to vize/@vizejs/native@0.600.0; native-template-global-policy.test.ts stays in tooling.                                                                                                                                                                                             |
| `typed-context-undefined.yml`         | #10667 public-failure evidence; typed-context-wrapper-mappings.test.ts stays in tooling.                                                                                                                                                                                                            |
| `unicode-prop-projection.yml`         | #10573 packets; constructor/unicode tooling tests stay in the suite.                                                                                                                                                                                                                                |
| `vapor-literal-object-styles.yml`     | #10749/#10923 baseline evidence; *-successors.test.ts stay in tooling.                                                                                                                                                                                                                              |
| `vue2-ir-template.yml`                | Two-revision public-IR evidence; native-vue2-ir-*.test.mjs stay in tooling.                                                                                                                                                                                                                         |

#### One-off code removed with them

52 files that only these workflows ran, chosen from the tools-plan per-workflow list and re-checked with `git grep` so that no surviving file refers to them:

- the `template-alias-paired-11822` harness family (`tools/benchmarks/scripts/template-alias-paired-11822*`) and `mapper-capture-consumer-runtime.test.ts`
- the native-checker CPU decoders and their test
- the compiler-options, ambient-template-ref and narrowed-slot oracles
- the expression-overlap parser cases and frozen observers
- the native build-token observers and the typed-context replay
- the SFC parse-attribute capture/compare drivers and their bootstrap test
- the CSS source-map event resolver and its test
- the async-component feature-delta and its workflow test

Edited tests: `setup-wild-http-recovery.test.mjs` and `support/rust-cache-docs-wasm-consumers.mjs` drop the deleted workflows, and `unicode-prop-projection-custody.test.ts` drops its workflow-registration block (its 19 custody tests stay). `retired-tooling-languages.json` loses the entries of the deleted `.mjs` files.

#### v1 evidence

`content-mapper-qualification.ts` / `docs/release/v1-supported-matrix.json`: the "Path, import and template aliases" row no longer cites `template-alias-paired-11822.yml`. It now cites `crates/vize/tests/check_canon_path_alias_json_cli.rs`, next to `check_default_imports_cli/aliased_diagnostics.rs`. `node tools/commands/release/v1-support-matrix.ts --check` passes.

#### Deferred

- `native-beta-diagnostic-dto.yml` stays. The legacy-fix fixture gate in `pr-source-checks` (`report-legacy-fix-fixtures.ts` → `native-beta-dto-fixtures.ts` and friends) pins that workflow's bytes through `workflow-program-history.json`. Deleting it needs that custody re-anchored first.
- `sfc-optional-lifecycle` and `sfc-parse-replay` are `criterion-bench` callees. They go in the criterion cleanup (part F).
- Shared harness code that a workflow still alive on `main` also uses (for example `tools/support/cross_file/async_component/verify.ts` and `tools/support/compat/defaulted-inject/verify.ts`, used by `public-native-events`) is left for the deletion of that last user.

#### Checks

- `node tools/commands/generate/repository-config.ts --check` (includes the catalog README)
- `node tools/commands/ci/verify-tool-layout.ts`
- `node --test` on workflow-layout, github-workflows*, release/*, retired-tooling-languages, v1-supported-matrix, tooling-test-selection and every edited test. The only failures are 2 in `release/verify-published-registry.test.ts`, which fail the same way on `main` (registry access).

CI/tooling-only change. No product source changes, so the product binaries are unchanged.

<!-- codesmith:footer -->

---

<a href="https://dashboardbackend.blacksmith.sh/track/run-proof?expires=1798088049&installation_model_id=422833&pr_number=13606&ref=codesmith_pr_footer&repository=ubugeeei-prod%2Fvize&return_to=https%3A%2F%2Fgithub.com%2Fubugeeei-prod%2Fvize%2Fpull%2F13606&variant=get_evidence&signature=6d83904cb45fd16fbe2a9eb0c179a0895fa5ba91c4f14451563ecd37962f4234"><picture><source media="(prefers-color-scheme: dark)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-get-evidence-v2-dark.svg"><source media="(prefers-color-scheme: light)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-get-evidence-v2-light.svg"><img alt="Get evidence" src="https://pr-comments-assets.blacksmith.sh/codesmith/proof-get-evidence-v2-dark-3x.png" width="123" height="28"></picture></a>
<sup><a href="https://app.blacksmith.sh/ubugeeei-prod/codesmith/vize/pr/13606?autoLogin=true&ref=codesmith_pr_footer">Open in [code]smith</a> · Need help on this PR? Tag <code>@codesmith-bot</code> with what you need.</sup>

<!-- codesmith:autofix:disabled -->
<!-- /codesmith:footer -->

</details>

<details><summary>#13609 — ci: delete editor and type-check evidence workflows</summary>

Part E of the workflow cleanup: delete the editor (group 05) and type-check / content-mapper (group 04) evidence workflows.

#### Deleted workflows (22)

Each one recorded evidence for one fix or one investigation: issue-numbered packets, published-host matrices, paired parent/child observations, cache and residency custody. `pr-source-cli.yml` also goes: its only caller was `named-sfc-value-hover`, and `pr-source-checks` uses `full-source-cli`. The guards that outlive them already run in the normal suite (vize integration and crate tests, plus the tooling tests that the affected-tooling and full-tooling lanes select).

| Workflow                                   | What it recorded / where the guard stays                                                                                                                                                                                                |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `checker-constructor-emitter.yml`          | Title-gated (refactor/perf(canon)) base/head parity; skipped 20/20; vize_canon constructor tests stay in the suite.                                                                                                                     |
| `component-input-diagnostic-ownership.yml` | #10607/#11153 packet evidence. The `callback-owner-*` certificate tests and `aria-*` tests stay; the workflow's own script/workflow tests go with it.                                                                                   |
| `component-input-projection-packets.yml`   | Same vize_maestro --features legacy davinci_ts40_projection test that content-mapper-conformance runs.                                                                                                                                  |
| `content-mapper-instance-globals.yml`      | #10617 capture. `content-mapper-instance-globals-10617.test.ts` (318 s) keeps running in nightly Full qualification through `dedicatedPrToolingTests`, and the other ~20 `content-mapper-instance-globals-*` tests stay in the PR lane. |
| `content-mapper-vue-component-meta.yml`    | #13430 fix evidence; its vize integration tests run in the normal suite.                                                                                                                                                                |
| `content-mapper-worker-stack.yml`          | Worker-stack calibration evidence; its calibration scripts and their test go with it (no product code involved).                                                                                                                        |
| `css-hover-visual.yml`                     | #10562/#10572 visual evidence; vscode-css-hover-_.test.mjs and lsp-css-_ tests stay in tooling.                                                                                                                                         |
| `lsp-jsx-intrinsic-hover.yml`              | Observation packets; lsp-jsx-intrinsic-hover.test.ts stays in tooling.                                                                                                                                                                  |
| `lsp-native-diagnostic-coverage.yml`       | Scheduler-law evidence; the affected-tooling capacity-packet assertion in `lsp-diagnostic-capacity-workflow.test.ts` stays.                                                                                                             |
| `lsp-native-diagnostic-warmup.yml`         | #10604 old-drop reproduction; tests stay in tooling.                                                                                                                                                                                    |
| `lsp-overlay-custody.yml`                  | #11145 custody evidence; default vize_canon tests run in the suite.                                                                                                                                                                     |
| `lsp-overlay-graph-reuse.yml`              | Parent/child paired observations.                                                                                                                                                                                                       |
| `lsp-residency-pair.yml`                   | Exact-parent residency pair (5/12 green); its report script and test go with it.                                                                                                                                                        |
| `lsp-scheduler-cost-carrier.yml`           | Micro-profiling cost-carrier census for one change; its workflow test goes with it.                                                                                                                                                     |
| `lsp-tag-prefix-classification.yml`        | #10622 observations; tests stay in tooling.                                                                                                                                                                                             |
| `named-sfc-value-hover.yml`                | #11953 packets; lsp-named-sfc-value-hover.test.ts stays in tooling.                                                                                                                                                                     |
| `pr-source-cli.yml`                        | Reusable producer whose only caller is named-sfc-value-hover (deleted); pr-source-checks uses full-source-cli instead. Orphan after that deletion.                                                                                      |
| `router-editor-artifact.yml`               | Installed router VSIX artifact evidence; 0/9 green.                                                                                                                                                                                     |
| `router-experimental-navigation.yml`       | Published router navigation matrix (published host).                                                                                                                                                                                    |
| `router-preview-conformance.yml`           | #10579 router preview provider evidence. The page oracle (`check-router-page-types`) still runs on full-tooling shard 1; `router-page-workflow.test.ts` now pins that path.                                                             |
| `typechecker-cache-custody.yml`            | Cache custody evidence vs a public baseline; its benchmark scripts and tests go with it.                                                                                                                                                |
| `vue2-editor-diagnostic-context.yml`       | Vue 2 rename/edit-cycle evidence; vue2-rename-context-custody.test.mjs stays in tooling.                                                                                                                                                |

#### One-off code removed with them

33 files that only these workflows ran, chosen from the tools-plan per-workflow list and re-checked with `git grep` so that no surviving file refers to them:

- the typechecker cache-custody scripts and their tests
- the content-mapper worker-stack scripts and their test
- the LSP residency-pair report
- the component-input ownership profiles and their diagnostic-ownership oracles
- `admission-hosts.ts` and `resolve-consumer-host.ts`
- the Router editor host harness (`editors/vscode/test/router-host-workspace.mjs`, `run-router-host-real.mjs`) and its contract test
- the workflow-only tests of the deleted workflows
- the stray root driver directory `.github/scripts/component-input-diagnostic-ownership/` and its Pkl source

Edited tests:

- `router-page-workflow.test.ts` now pins the page oracle where it still runs: `check-router-page-types` on full-tooling shard 1, with the same evidence upload paths.
- `lsp-diagnostic-capacity-workflow.test.ts` keeps its affected-tooling capacity-packet assertion.
- `setup-wild-http-recovery.test.mjs` drops the deleted workflow.

`plan-tooling-tests.mjs`: `dedicatedPrToolingTests` keeps `content-mapper-instance-globals-10617.test.ts` (318 s in the timing table), so it still runs only in nightly Full qualification and stays off the PR critical path. Only the comment that pointed at the deleted dispatchable workflow changed.

#### Not covered here (left for the tools sweep)

- The partly shared `callback-owner-*` and `admission-hosts-*` content-mapper modules. Surviving certificate tests still import them.
- `tests/tooling/support/lsp/native-warmup-witness.ts`. The warm-up tests still use it.
- The CSS hover visual harness under `editors/vscode/test/` and its `vscode-css-hover-*.test.mjs` tests. They still run in tooling and the inventory keeps them.

#### Checks

- `node tools/commands/generate/repository-config.ts --check` (includes the catalog README)
- `node tools/commands/ci/verify-tool-layout.ts`
- `node tools/commands/release/v1-support-matrix.ts --check`
- `node --test` on workflow-layout, github-workflows*, release/*, retired-tooling-languages, v1-supported-matrix, tooling-test-selection and every edited test. The only failures are 2 in `release/verify-published-registry.test.ts`, which fail the same way on `main` (registry access).

CI/tooling-only change. No product source changes, so the product binaries are unchanged.

<!-- codesmith:footer -->

---

<a href="https://dashboardbackend.blacksmith.sh/track/run-proof?expires=1798088067&installation_model_id=422833&pr_number=13609&ref=codesmith_pr_footer&repository=ubugeeei-prod%2Fvize&return_to=https%3A%2F%2Fgithub.com%2Fubugeeei-prod%2Fvize%2Fpull%2F13609&variant=run_proof_on_this_pr&signature=17183a3db65159131ff54c3ad0ab37e76c180131f60ecdcfa25a269c7e495b17"><picture><source media="(prefers-color-scheme: dark)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-run-proof-on-this-pr-v2-dark.svg"><source media="(prefers-color-scheme: light)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-run-proof-on-this-pr-v2-light.svg"><img alt="Run Proof on this PR" src="https://pr-comments-assets.blacksmith.sh/codesmith/proof-run-proof-on-this-pr-v2-dark-3x.png" width="168" height="28"></picture></a>
<sup><a href="https://app.blacksmith.sh/ubugeeei-prod/codesmith/vize/pr/13609?autoLogin=true&ref=codesmith_pr_footer">Open in [code]smith</a> · Need help on this PR? Tag <code>@codesmith-bot</code> with what you need.</sup>

<!-- codesmith:autofix:disabled -->
<!-- /codesmith:footer -->

</details>

<details><summary>#13607 — ci: delete performance and remaining evidence workflows</summary>

#### Summary

Part of the workflow cleanup (plan PR G): delete one-off evidence workflows from groups 01, 03 and 07–11. These are paired base/head profiles, allocation and cost carriers, reproductions of reported or published projects, and per-change admission packets. Their drivers, Pkl sources and Pkl script directories go, along with the `tools/` harnesses and tests that served only them. Every file was re-checked with `git grep` to have no surviving referrer.

- Workflows: **45 deleted** (280 → 235 on its own)
- Other files deleted: **75** (`tools/benchmarks/scripts/*`, `tools/support/compat/*`, `tools/benchmarks/crates/vize/tests/*_profile.rs`, and the tests that only pinned these harnesses or workflows)
- Edited: 7 mixed tests lose their blocks for the deleted workflows. The stale keys go from `tooling-test-timings.json` and `retired-tooling-languages.json`, and the catalog README is regenerated.

#### Deleted workflows and why the guard is still covered

| Workflow                                     | Group | Reason / where the lasting guard runs                                                                                                                      |
| -------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `canon-component-casing.yml`                 | 07    | Accuracy evidence (published false negatives).                                                                                                             |
| `canon-helper-native-baseline.yml`           | 07    | Frozen public repro and poison evidence.                                                                                                                   |
| `canon-instance-directives.yml`              | 07    | Merge-parent red/candidate packets for one change.                                                                                                         |
| `canon-slot-constructor.yml`                 | 07    | #11013/#11023 "original27" admission.                                                                                                                      |
| `canon-slot-provider-any.yml`                | 07    | #11027 reviewed-source admission.                                                                                                                          |
| `component-binding-view-profile.yml`         | 03    | Per-change allocation profile (merge-parent pair); 4/11 green.                                                                                             |
| `component-scope-binding.yml`                | 03    | Per-change allocation profile (merge-parent pair).                                                                                                         |
| `content-mapper-span-allocation.yml`         | 03    | Per-change before/after allocation evidence; 5/12 green.                                                                                                   |
| `cross-file-finalization-compare.yml`        | 07    | Paired optimized-provider work comparison.                                                                                                                 |
| `cross-file-finalization-order.yml`          | 07    | Base/head ordering observers; scope tests stay in tooling.                                                                                                 |
| `cross-file-source-updates.yml`              | 07    | Retained-owner regression packets; vize_croquis_cf tests run in the suite.                                                                                 |
| `cross-file-wasm-effects.yml`                | 07    | Same vize_vitrine wasm effect tests as cross-file-finalization (moved to the feature lane).                                                                |
| `davinci-shared-toolchain-session.yml`       | 07    | Replays original editor revisions and compares against an ancestor.                                                                                        |
| `davinci-unused-bindings.yml`                | 07    | P0-13 pilot profile comparison; davinci-unused-* tooling tests stay in the suite.                                                                          |
| `diagnostic-digest-bench.yml`                | 03    | Cost evidence for the diagnostic-digest change; PR trigger only registers it.                                                                              |
| `doctor-barrel-props.yml`                    | 09    | Baseline-reproduction packets; vize doctor tests run in the suite.                                                                                         |
| `doctor-lint-ssr-context.yml`                | 09    | Same baseline-reproduction template.                                                                                                                       |
| `doctor-prop-accuracy.yml`                   | 09    | Same baseline-reproduction template.                                                                                                                       |
| `document-snapshot-source-profile.yml`       | 03    | micro-profiling carrier for one change.                                                                                                                    |
| `full-worker-contracts.yml`                  | 01    | #7502 cancellation-contract harness; only runs tests/tooling/github-workflows-*.test.ts which affected tooling already selects when workflow files change. |
| `fuzz-expression-10401-profile.yml`          | 03    | Issue-numbered (#10401) slow-unit profile.                                                                                                                 |
| `glibc-floor-bench.yml`                      | 03    | Decision evidence for building at the glibc floor; 02-release-glibc-floor is the gate.                                                                     |
| `js-install-closure.yml`                     | 08    | js-install-dependency-closure.test.mjs stays in tooling.                                                                                                   |
| `lint-unused-disable-config.yml`             | 09    | lint-unused-disable-config.test.ts stays in tooling.                                                                                                       |
| `lsp-marker-scan-scale.yml`                  | 03    | Base/head marker-scan allocation comparison for one change.                                                                                                |
| `lsp-source-profile.yml`                     | 03    | Instrumented profiling evidence; its resource helper tests are in tests/tooling.                                                                           |
| `mounted-runtime-devtools-comparison.yml`    | 10    | Transition trace comparison; mounted-runtime-*.test.ts stay in tooling.                                                                                    |
| `packed-browser-equivalence.yml`             | 11    | Base/head unpacked-API comparison; packed-browser-equivalence.test.ts stays in tooling.                                                                    |
| `partition-weight-work.yml`                  | 03    | Old/new shard-plan law evidence; partition tests are in tests/tooling and vize_canon.                                                                      |
| `patina-attribute-ignore-tags-reference.yml` | 09    | #6097 reference capture; last run 11-01.                                                                                                                   |
| `patina-native-boolean-protocol.yml`         | 09    | "Exact reported project runtime" evidence; default vize/vize_patina tests run in the suite.                                                                |
| `persistent-batch-storage.yml`               | 03    | #10934 storage evidence; nextest targets are in the normal suite.                                                                                          |
| `plain-script-import-ref.yml`                | 08    | #10903/#11885 public-0.589 before/after; plain-script-project-metadata-reference.test.ts stays in tooling.                                                 |
| `qs-runtime-security.yml`                    | 11    | One-off qs advisory repair evidence for the MCP server; check security-audit runs pnpm audit.                                                              |
| `script-recovery-clone-cost.yml`             | 03    | Paired base/head clone-cost evidence.                                                                                                                      |
| `script-recovery-cost.yml`                   | 03    | Paired base/head evidence; 0/12 green.                                                                                                                     |
| `setup-binding-anchor-profile.yml`           | 03    | Per-change allocation profile.                                                                                                                             |
| `source-operation-profile.yml`               | 03    | Collector-overhead evidence for one change.                                                                                                                |
| `string-storage-bench.yml`                   | 03    | Paired base/head storage benchmark for one optimization; criterion A/B covers re-measurement.                                                              |
| `text-boundary-bench.yml`                    | 03    | Measures one design question (UTF-8 validation vs read cost) with no budget; criterion A/B covers re-measurement.                                          |
| `type-admission-scale.yml`                   | 03    | Original public-project axis evidence (vize-version input).                                                                                                |
| `type-snapshot-bench.yml`                    | 03    | Paired snapshot benchmark for one change; last green 10-20.                                                                                                |
| `vite-style-relative-imports.yml`            | 08    | Untouched reported consumer reproduction.                                                                                                                  |
| `windows-config-consumer.yml`                | 09    | Public before/after phases for one fix; release-verify already runs the Windows JS-config command from public npm.                                         |
| `workspace-route-identity.yml`               | 09    | #11265 laws; package-shadow-work-admission.test.ts stays in tooling.                                                                                       |

Kept on purpose, because each is a retained guard (the plan names these as staying in tooling): `js-install-dependency-closure.test.mjs` (+ `js-install-closure.mjs`), `lint-unused-disable-config.test.ts`, `packed-browser-equivalence.test.ts` (+ `tools/commands/levels/packed-browser-equivalence.ts`, `wasm-responses.ts`), `mounted-runtime-*.test.ts`, `davinci-unused-*` tests, the partition and `package-shadow-work-admission` tests. `tools/support/launchers/exec.sh` stays because `process-launcher.ts` loads it by a dynamic name.

Measurement-only coverage that is dropped: `micro-profiling`/`profiling` feature runs and `--ignored` paired timings in the deleted profile workflows. Criterion and source-comparison A/B dispatches remain the way to re-measure.

#### Not in this PR

- `bundler-block-products` and `versioned-fix-publication`: these are the only CI for `vize_atelier_sfc --features bundler-block-products` and `vize_l0`/`vize_l1 --features positioned-edits(,diagnostic-edit-selection)`. They move to the experimental Rust feature lane first.
- `language-project-fixtures`: `language-project-classification-runtime.ts` byte-pins its workflow file in a custody chain that is still alive. It needs its own change.
- `sfc-member-profile`: a `criterion-bench` callee, removed with the criterion inputs.
- `MERGE-INTO` / `MOVE-TO-TESTS` rows: separate PRs.
- Tools files shared with workflows that still exist (e.g. `typechecker-native-*`, `canon-slot-provider-cli`, `type-snapshot-cli-*`, `content-mapper/*-transition.ts`, `windows_consumer/*`): they go when their last consumer goes.

#### Product impact

CI/tooling-only. No file under `crates/`, `davinci/` or `npm/*/src` changes, so every product binary (`vize`, `@vizejs/native`, the VSIX and the WASM) is unchanged.

#### Checks run locally

- `node tools/commands/generate/repository-config.ts --check`
- `node tools/commands/ci/verify-tool-layout.ts`
- `node --test tests/tooling/workflow-layout.test.ts tests/tooling/github-workflows*.test.* tests/tooling/release/*.test.ts` plus every edited test, `retired-tooling-languages`, `tooling-test-timings/selection/shards`: 801 tests. 13 failures also fail on `main` on macOS: `setup-wild-http-recovery` needs GNU `sha256sum`, and `verify-published-registry` depends on the live registry.

<!-- codesmith:footer -->

---

<a href="https://dashboardbackend.blacksmith.sh/track/run-proof?expires=1798088061&installation_model_id=422833&pr_number=13607&ref=codesmith_pr_footer&repository=ubugeeei-prod%2Fvize&return_to=https%3A%2F%2Fgithub.com%2Fubugeeei-prod%2Fvize%2Fpull%2F13607&variant=see_it_in_action&signature=32ed068291b237e98470066c0fceec5f826f8d1bc89a9ca985753da29e40f2ab"><picture><source media="(prefers-color-scheme: dark)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-see-it-in-action-v2-dark.svg"><source media="(prefers-color-scheme: light)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/proof-see-it-in-action-v2-light.svg"><img alt="See it in action" src="https://pr-comments-assets.blacksmith.sh/codesmith/proof-see-it-in-action-v2-dark-3x.png" width="132" height="28"></picture></a>
<sup><a href="https://app.blacksmith.sh/ubugeeei-prod/codesmith/vize/pr/13607?autoLogin=true&ref=codesmith_pr_footer">Open in [code]smith</a> · Need help on this PR? Tag <code>@codesmith-bot</code> with what you need.</sup>

<!-- codesmith:autofix:disabled -->
<!-- /codesmith:footer -->

</details>

<details><summary>#13608 — ci: run experimental Rust feature tests in one lane</summary>

#### Summary

Twenty-five pull-request workflows were the only CI for experimental Cargo features: every workspace suite builds default features, plus `native-lex-parity,legacy` for `vize_armature` only. This PR moves their tests and Clippy runs into one lane. Then it deletes the workflows, or keeps only their proof jobs.

- **Manifest**: `tools/support/ci/rust-feature-tests.ts` has 33 rows `{id, packages, commands, env, windows, runtime}`. The runner `node tools/commands/ci/rust-feature-tests.ts run <row>` adds `--locked`, `--profile ci` (unless the command names a profile) and `-- -D warnings` for Clippy.
- **PRs and the merge queue**: `pr-rust-products` gains a `rust-feature-tests` matrix job. `plan-pr-checks.ts` selects the rows whose crates fall in the actual affected-crate closure. An unknown diff or an edit to the lane itself selects every row, so this PR runs all of them. `rust-product-report` gates on the job. `require-rust-tier.ts --products` accepts a skip only when the planner selected no rows (`featureTests == false`).
- **Full qualification**: calls the new reusable `01-rust-feature-tests.yml` (a hosted plan job plus the same matrix job), which runs every row on schedule and dispatch.
- Job names are unchanged. `check.yml` is untouched.

#### Rows (source workflow → lane row)

| Former workflow               | Lane row(s)                                                                                                                                                                             |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| art-source-products           | `art-sources`                                                                                                                                                                           |
| canon-windows-mirror-paths    | `windows-project-paths` (Windows runner, locked TypeScript runtime). The Linux leg ran default-feature tests that the workspace suite already runs.                                     |
| checker-diagnostic-positions  | `diagnostic-positions`                                                                                                                                                                  |
| checker-styled-projections    | `styled-projections`                                                                                                                                                                    |
| client-server-decisions       | `client-server-decisions`                                                                                                                                                               |
| cross-file-finalization       | `script-binding-products` (`vize_vitrine --no-default-features --features wasm` effect tests). The default-feature `vize`/`vize_croquis_cf` filters already run in the workspace suite. |
| cross-file-sfc-effects        | `script-binding-products`                                                                                                                                                               |
| davinci-ir-graph              | `graph-queries`. The workflow keeps its `models` job (Lean, Alloy, TLC).                                                                                                                |
| declaration-source-products   | `declaration-products`, `literal-products`. The workflow keeps `primitive-assignability-proof` (Kani, Alloy).                                                                           |
| editor-script-queries         | `editor-script-queries` (the `vize_maestro` products), `script-query-tools` (`vize_l1`, `vize_glyph`, `vize_patina`, `vize_canon`)                                                      |
| ir-pass-composition           | `template-passes`, `resident-template-passes`. The workflow keeps `lint-ranking` and `partition` (Lean, Kani).                                                                          |
| jsx-authored-native-modules   | `jsx-module-projection` (`component-projection-tests`, `module-ir`). The default-feature `vize_atelier_jsx` targets already run in the workspace suite.                                 |
| lsp-config-diagnostics        | `legacy-language-server` (`--features legacy`/`profiling` for `vize`, `vize_canon`, `vize_maestro`)                                                                                     |
| musea-glyph-feature-boundary  | `cli-feature-sets`                                                                                                                                                                      |
| native-image-alt-query        | `image-alt-rule-group`                                                                                                                                                                  |
| native-navigation-products    | `navigation-products`                                                                                                                                                                   |
| open-document-blocks          | `open-document-blocks`                                                                                                                                                                  |
| plugin-template-products      | `plugin-template-products`                                                                                                                                                              |
| production-reach-accounting   | `production-reach`                                                                                                                                                                      |
| profile-allocation-thread-law | `profile-allocations`. The profiling CLI compile moves to `cli-feature-sets`.                                                                                                           |
| retained-script-formatting    | `retained-script-formatting`                                                                                                                                                            |
| retained-tsrx                 | `retained-tsrx`                                                                                                                                                                         |
| style-module-products         | `style-modules`, `scss-products`. The workflow keeps `authored-coordinates` (Kani, Lean).                                                                                               |
| template-expression-queries   | `component-listeners`, `component-props`, `lexical-arrows`, `expression-queries`                                                                                                        |
| template-tool-projections     | `template-tool-projections`                                                                                                                                                             |
| bundler-block-products        | `bundler-blocks`                                                                                                                                                                        |
| versioned-fix-publication     | `versioned-fixes`                                                                                                                                                                       |

#### Dropped on purpose (per the maintainer decision on `--ignored` tests)

The lane runs no `--ignored` test, and an `--include-ignored` filter keeps only its non-ignored tests. The ignored tests stay in the code and can still be run by hand. They read corpora that the deleted workflows selected outside the repository ("original" files and Vben/Element Plus checkouts), or they measure time and allocations.

Ignored tests the lane no longer runs, by former workflow:

| Former workflow              | Ignored tests (crate / target / filter)                                                                                                                                                                                                                                                                     |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| art-source-products          | `vize_musea` `art_source_products`: `descriptor_warnings_and_complete_gallery_products_on_original_files`, `complete_responses_share_the_original_art_surface`                                                                                                                                              |
| checker-diagnostic-positions | `vize_l4` `program_projection`: `diagnostic_positions::original_projected_diagnostics`, `diagnostic_positions::profile::paired_queries`                                                                                                                                                                     |
| checker-styled-projections   | `vize_canon` `styled_projection`: `original_styled_files`; ignored tests in `original_style_bindings`                                                                                                                                                                                                       |
| client-server-decisions      | ignored tests in `vize_atelier_sfc` `client_server_decisions`                                                                                                                                                                                                                                               |
| cross-file-sfc-effects       | ignored tests under `compile_script::binding_products::`                                                                                                                                                                                                                                                    |
| davinci-ir-graph             | `vize_l2_to_l3` `native_ir_corpus`; graph provider `source_revisions`; the `source_sites`/`recovery_sites` example measurements                                                                                                                                                                             |
| declaration-source-products  | `vize_maestro`: `original_declaration_products_match`, `paired_declaration_products`; ignored tests under `corsa::ambient_project::`, `plain_exports::observed::`, `source_project::literal_products::`, `source_project::recovered_script::`                                                               |
| editor-script-queries        | ignored tests under `source_project::script_queries::`, `script_parse::`, `authored_script_rules`                                                                                                                                                                                                           |
| ir-pass-composition          | ignored tests in `grouped_passes`, `original_template_queries`, `observed_template_passes`, `cfg_complexity_oracle composed::`, `original_template_lint`, `resident_template_cost`, `template::tests::`, `source_project::template::`                                                                       |
| jsx-authored-native-modules  | `vize_canon` `component_prop_projection`: `corpus::component_props_in_original_modules`; `vize_l4` `jsx_module_projection`: `neutral_corpus::compare_modules_at_every_outcome`                                                                                                                              |
| native-image-alt-query       | `vize_patina` `canonical_rule_group` / `ci_canonical_lint_probe`: `original_sources`, `regions::original_region_events`, `profile::{individual_lint_requests,paired_lint_requests,original_request_costs}`                                                                                                  |
| native-navigation-products   | `vize_maestro`: `original_file_membership_and_public_navigation_packets_match`, `paired_original_navigation_requests`                                                                                                                                                                                       |
| open-document-blocks         | `vize_maestro`: `source_project::blocks::tests::original_editor_documents`                                                                                                                                                                                                                                  |
| plugin-template-products     | `vize` `plugin_template_products`: `original_packets_and_component_records_match`, `measure_complete_*`; ignored tests in `inspector::`, `inspector_template_products`                                                                                                                                      |
| retained-script-formatting   | `vize_glyph`: `script::snapshot::tests::corpus::original_sources`, `…::profile::{paired_formatter_requests,corpus::original_formatter_requests}`; `selected_script_programs`: `original_sources`, `profile::{shared_program_requests,corpus::original_program_requests}`                                    |
| retained-tsrx                | ignored tests in `vize_patina` `prepared_tsrx`                                                                                                                                                                                                                                                              |
| style-module-products        | `vize_maestro`: `original_style_{lint,module}_products_match`, `paired_style_{lint,module}_requests`; ignored tests in `scss_products`, `native_sfc_scss`, `observed_style`, `source_project::{scss,style_format}::`                                                                                        |
| template-expression-queries  | `vize_l1_to_l2` `expression_queries`: `corpus::original_vue_files`, `component_imports::{original_vue_files,paired_queries}`, `operand::paired_operand_products`, `profile::paired_reference_queries`; `vize_maestro`: `original_event_documents`; `vize_croquis_cf`: `facts::render_tree::setup::tests::*` |
| template-tool-projections    | `vize_maestro`: `original_tool_template_projections_match`                                                                                                                                                                                                                                                  |
| bundler-block-products       | ignored tests in `bundler_block_products`                                                                                                                                                                                                                                                                   |
| versioned-fix-publication    | `versioned_fix_publication`: `original_lint_fixes`, `profile::publication_cost`                                                                                                                                                                                                                             |

Also dropped: base/head and published-version comparisons, packet/evidence uploads, `cargo run --example` profiles (their Clippy builds stay where the workflow had them), `profile_guard_send_law.ts` (old-versus-new guard comparison), the `ir-pass-composition` `stock` shipping-entry comparison, and the node oracles that consumed the corpus outputs (`primitive-assignments-oracle.ts`, `literal-hover-oracle.ts`, `select-broken-scripts.ts`, `run-lsp-config-diagnostics.ts`).

#### Evidence

This PR edits the lane, so the planner selected all 33 rows. Every row passed on the first PR run, [45329598659](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659) (33/33). The Windows row ran on `blacksmith-32vcpu-windows-2025`.

| Row                          | Result                                                                                  |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| `art-sources`                | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492572) |
| `bundler-blocks`             | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492487) |
| `cli-feature-sets`           | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492454) |
| `client-server-decisions`    | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492494) |
| `component-listeners`        | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492457) |
| `component-props`            | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492531) |
| `declaration-products`       | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492720) |
| `diagnostic-positions`       | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492548) |
| `editor-script-queries`      | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492486) |
| `expression-queries`         | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492568) |
| `graph-queries`              | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492554) |
| `image-alt-rule-group`       | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492509) |
| `jsx-module-projection`      | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492634) |
| `legacy-language-server`     | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492671) |
| `lexical-arrows`             | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492681) |
| `literal-products`           | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492628) |
| `navigation-products`        | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492593) |
| `open-document-blocks`       | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492575) |
| `plugin-template-products`   | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492642) |
| `production-reach`           | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492612) |
| `profile-allocations`        | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492636) |
| `resident-template-passes`   | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492590) |
| `retained-script-formatting` | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492667) |
| `retained-tsrx`              | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492712) |
| `script-binding-products`    | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492639) |
| `script-query-tools`         | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492945) |
| `scss-products`              | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492851) |
| `style-modules`              | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492721) |
| `styled-projections`         | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492730) |
| `template-passes`            | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492719) |
| `template-tool-projections`  | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492678) |
| `versioned-fixes`            | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492652) |
| `windows-project-paths`      | [pass](https://github.com/ubugeeei-prod/vize/actions/runs/45329598659/job/138227492774) |

`rust-product-report` passed with the new `rust-feature-tests` dependency. A later commit moves the full-qualification copy of the lane into the reusable `01-rust-feature-tests.yml` to keep `full-qualification.yml` under the 500-line source limit. The PR-side lane is unchanged.

CI capacity: on an ordinary PR the lane runs only the rows whose crates the affected-crate closure reaches, which replaces 25 path-filtered workflows with 1–6 jobs each. Full qualification adds 33 jobs per scheduled or dispatched run.

#### Products

CI and tooling only. No crate source changes, so the `vize` binary and every product binary are unchanged.

#### Follow-ups

- The proof jobs left in davinci-ir-graph, declaration-source-products, ir-pass-composition and style-module-products join `07-proofs` in the proofs merge (I3).
- `tests/tooling/support/cross-file-{finalization,sfc-effects}.ts` and `tools/support/compat/npm/bundler-block-products.ts` / `tools/commands/levels/native-addon-identity.ts` are still referenced by workflows that a separate deletion PR removes. They become orphans when both PRs land.

</details>

<details><summary>#13626 — ci: strip one-off modes from kept workflows</summary>

#### Summary

Part of the workflow cleanup (PR F). It removes the one-off modes, jobs and inputs that kept per-fix evidence inside workflows we keep. 201 files changed, about 22k lines deleted.

| Kept workflow                 | Removed                                                                                                                                                                                                                | Why the guard is still covered                                                                                                                                                                                                                                                                                                                                                                                           |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Content Mapper Conformance    | The `reported-q16` callee workflow (`content-mapper-reported-q16.yml`); the router-experimental, event-rename and official-css-options jobs; the `published-*` inputs (report, cut, head, tag, source PR, release run) | The daily `exact-tsgo-project` job is unchanged: exact upstream tsgo, content-mapper protocol regressions, project/declaration conformance, the LSP/overlay editor checks and the TS 4.0 projection baselines. The `generic-body-error` targeted mode stays. The v1 `css-modules` row now cites `tests/tooling/canon-css-module-contracts.test.ts` alone; the deleted `official-css-options.ts` was the other candidate. |
| Benchmark - Criterion suites  | The `sfc_optional_lifecycle_only`, `sfc_member_only` and `historical_sfc_parse` inputs and their callee workflows `sfc-optional-lifecycle.yml`, `sfc-member-profile.yml` and `sfc-parse-replay.yml`                    | The Criterion A/B, dialect guard, production pairs, Vapor mode and Level memory mode are unchanged                                                                                                                                                                                                                                                                                                                       |
| Release contracts             | The `public-release-contracts`, `public-custom-properties` and `type-admission-source-capability` dispatch jobs (duplicates of the public-native-events one-offs) and all their dispatch inputs                        | The `contracts` job (release workflow, npm and editor publication tests) runs on the same PR paths and on dispatch                                                                                                                                                                                                                                                                                                       |
| Benchmark - CI wall time      | The `js-native-fault-diagnostic` forensic job and the `n8n-baseline` / `n8n-preparation` components                                                                                                                    | The js, docs, rust, native, release-builds, nix and artifact-payload comparisons stay. The PR n8n qualification keeps its own partitioned-mode test (`github-workflows-n8n-baseline-mode.test.mjs`).                                                                                                                                                                                                                     |
| Benchmark - Source comparison | The campaign-only `patina_project_batch`, `patina_plain_type_packets` and `check_diagnostic_messages` inputs, their steps and job                                                                                      | The weekly and dispatch base-vs-head comparison and the PR budget are unchanged                                                                                                                                                                                                                                                                                                                                          |
| Vite+ configuration consumers | The `published-contracts` / `published-startup` jobs (published 0.75/0.76 startup reports)                                                                                                                             | The source CLI consumer job is unchanged                                                                                                                                                                                                                                                                                                                                                                                 |
| n8n adoption                  | The `compiler-custody` before/after job (#8142)                                                                                                                                                                        | The fixture and authored-editor jobs are unchanged                                                                                                                                                                                                                                                                                                                                                                       |

Tooling deleted with these modes, each file checked with `git grep` for surviving referrers:

- `reported-q16*` content-mapper harness, `official-css-options.ts`
- `sfc-member-*`, `sfc-optional-lifecycle*` and `sfc-parse-replay*` benchmark scripts
- `patina-project-batch*`, `patina-plain-type-packets*` and `check-diagnostic-messages*` scripts and fixtures
- `verify_published/vite_startup*`, the n8n compiler custody helpers
- the orphaned drivers of each workflow
- the tests that only pinned these modes, with their timing keys

Byte pins: the `workflow-program-history.json` digests and closures for `content-mapper-conformance` and `release-contracts` are refreshed. The archived predecessors stay unchanged. The `content-mapper-reported-q16` entry and its `.yml.gz` archive are removed.

With #13604 merged, the `public_*` release harnesses that only public-native-events and the stripped release-contracts jobs used (`public_icons`, `public_canonical_sdk`, `public_type_admission`, `public_release_contracts`, `public_custom_properties`) and `type-admission-cpp-custody.ts` / `type-admission-eligibility.ts` have no referrer, so this PR deletes them too. `human_reports_11252_11253_public` still has its own unit tests and is left for the tools sweep.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `node tools/commands/release/v1-support-matrix.ts --check`, `node tools/commands/ci/verify-tool-layout.ts`.
- `node --test` over `workflow-layout`, `github-workflows*`, `content-mapper-*`, `ci-walltime*`, `n8n*`, `official-*`, `criterion*`, `benchmark*`, `viteplus*`, `v1-*` and `release/*`: 1,200 tests pass. The 6 local failures need a source-built CLI or the network (`n8n-authored-editor`, `viteplus-*-cli`, 2 cases in `release/verify-published-registry`, which fail the same way on main), or were fixed afterwards (`release-notes`, re-run green). The release contract tests now read the moved list through `readAutomationPrograms`.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
