## Summary

The second combined workflow-cleanup PR, following #13633: the workflow merges (I1–I3) plus the inline-command guard (K). It starts at 125 workflows after #13633 and leaves 107.

- **Merges into host workflows (I1–I3):**
  - The corpus, content-mapper pair and replay harnesses become modes of `typecheck-cold-pair`.
  - The shipped-profile LSP resource comparison becomes a job of `davinci-resource-budgets`.
  - The three Chromium oracles move into `level-html-content-model`.
  - Nuxt generated membership becomes a job of content-mapper conformance.
  - Creusot and source-frames move into the Kani proofs workflow. It also gains the two Creusot paths that #13618 added on main.
  - Rspack, Vitest and Vite+ lint/task-cache become jobs of their hosts.
- **Deleted outright:**
  - `cold-typechecker-scale`: its base/head cold A/B is the host's default mode. The lead accepted dropping its PR-time profiling, since dispatch still covers it.
  - `vue-benchmarks-current-qualification`: 0/11 green.
  - `lsp-focused-projects`: covered by the real-project matrix and the current-source LSP baseline.
  - `lsp-dual-server-benchmark`: a duplicate of `lsp-vs-volar`.
- **Inline-command guard (K):**
  - The generator now rejects a `run` longer than 160 characters, a `node --test` step with more than two inline files, and an `if` longer than 200 characters.
  - The steps that already break these limits are listed in `automation-size-legacy.ts`; nothing may join that list.
  - Protected release files are exempt.
  - 29 long commands moved into named, wrapped Pkl programs. A script checked that each body equals the old command once its line continuations are joined.
- **Byte pins:** refreshed for `content-mapper-conformance` and `release-contracts`.

## Verification (local)

- Generator `--check`, `v1-support-matrix --check`, `verify-tool-layout`, `vp check` and the source-length check all pass.
- `node --test` over 144 affected tooling files: 790 pass and 13 fail; the same 13 fail on main in this environment.

## Product impact

CI and tooling only. No crate, `davinci/` or npm package source changes, so the product binaries are unchanged.

<details><summary>I1 — merge benchmark workflows</summary>

#### Summary

Workflow cleanup, PR I1: merge the benchmark workflows. 7 workflow files are removed (5 merged into hosts, 2 deleted), plus 58 benchmark scripts and tests that served only the deleted ones.

| Workflow                             | Action                                                                                   | Where the guard lives now                                                                                                                                                                                                                                                                                                  |
| ------------------------------------ | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| typecheck-corpus-pair                | Merged as mode `corpus` of `typecheck-cold-pair` (job `corpus`)                          | Same steps and drivers (`corpus-*`). `base_sha`/`head_sha` become the shared `base`/`head` inputs.                                                                                                                                                                                                                         |
| content-mapper-pair                  | Merged as mode `content-mapper` (jobs `mapper-*`)                                        | Same Linux, WSL2 and Windows cells and drivers (`mapper-*`)                                                                                                                                                                                                                                                                |
| content-mapper-replay                | Merged as mode `content-mapper-replay` (jobs `replay-resolve`, `replay-build`, `replay`) | Same jobs. The `bench/content-mapper-replay/**` push trigger moves to the host.                                                                                                                                                                                                                                            |
| lsp-release-resources                | Inlined as job `release-resource` of `davinci-resource-budgets`                          | Same steps behind the same `include_release` input. The duplicate toolchain driver reuses the host's identical `install-the-repository-rust-toolchain-2.sh`.                                                                                                                                                               |
| cold-typechecker-scale               | Deleted                                                                                  | Its base/head cold comparison is the `generated` mode of the same host (`projects: vue:2500`), and its PMU/flamegraph profiling is `typecheck-profile`. Its PR-time profile run on `default_imports.rs`/`dependency_scan` changes is dropped. Byte pin and `.yml.gz` archive removed from `workflow-program-history.json`. |
| vue-benchmarks-current-qualification | Deleted (0/11 green; the plan allowed "fold or drop")                                    | The corpus stays measured by `vue-benchmarks-replay`                                                                                                                                                                                                                                                                       |

Host details (`typecheck-cold-pair`, the future `03-typecheck-compare`):

- A new `mode` input (`generated` default, so existing dispatches behave the same) gates each mode's root jobs. The union of inputs is 25, which is GitHub's limit.
- Moved jobs live in `tools/config/pkl/parts/typecheck-cold-pair/`. Their drivers sit in the host directory with a mode prefix.
- The PR `test` job runs all three contract files: `typecheck-cold-pair.test.ts`, `content-mapper-pair.test.ts`, `content-mapper-replay.test.ts`.
- The v1 `cold-warm-edit` evidence row now cites `.github/workflows/typecheck-cold-pair.yml`; `docs/release/v1-supported-matrix.json` is regenerated.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `node tools/commands/ci/verify-tool-layout.ts`, `node tools/commands/release/v1-support-matrix.ts --check`.
- `node --test` over workflow-layout, github-workflows*, typecheck-cold-pair, content-mapper-pair/replay, typecheck-corpus-pair*, automation-text-filters, lsp-release-resources, github-source-cli-workflow, typecheck-owned-pair*, typecheck-vs-vue-tsc*, v1-supported-matrix, rust-cache-docs-* and retired-tooling-languages: 392 tests. The 11 failures are all in `setup-wild-http-recovery.test.mjs` and fail the same way on main locally. `davinci-resource-budgets.test.ts` has one local nix-warning failure that also fails on main.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

<details><summary>I2 — merge editor and browser-oracle workflows</summary>

#### Summary

Workflow cleanup, PR I2: merge the editor and browser-oracle workflows. 6 workflow files are removed.

| Workflow                   | Action                                                                                                      | Where the guard lives now                                                                                                                                                                                                                                                                                                                                                      |
| -------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| l1-document-special-text   | Merged as job `special-text` of `level-html-content-model`                                                  | Same steps and drivers (`special-text-*`)                                                                                                                                                                                                                                                                                                                                      |
| pre-textarea-first-newline | Merged as job `first-newline`                                                                               | Same steps (`first-newline-*`)                                                                                                                                                                                                                                                                                                                                                 |
| scoped-nesting-specificity | Merged as job `scoped-cascade`                                                                              | Same steps (`scoped-nesting-*`)                                                                                                                                                                                                                                                                                                                                                |
| nuxt-generated-membership  | Merged as scheduled job `nuxt-generated-membership` of `content-mapper-conformance`, without packet custody | Same `check_cli::check_nuxt_generated_computed_map_values_preserve_record_types` run with real tsgo and stock vue-tsc, plus `verify-membership-controls.ts`, which requires every native and stock control to exit with its reviewed status. The v1 `nuxt-generated-types` evidence row now names `content-mapper-conformance.yml`, and the conformance byte pin is refreshed. |
| lsp-focused-projects       | Deleted                                                                                                     | It was a bounded replay of four original real-project failures (1/7 green). The real-project matrix and `lsp-current-baseline` cover real-project LSP behavior.                                                                                                                                                                                                                |
| lsp-dual-server-benchmark  | Deleted as a duplicate                                                                                      | `lsp-vs-volar` is the one Vize-vs-Vue-tools comparison. The dual-LSP oracle modules and their tests stay because other tests import them. Only the benchmark entry point, its sampling helpers and the `test:performance:lsp-dual-server:controls` task go.                                                                                                                    |

`level-html-content-model`, the future `06-browser-oracles`:

- A hosted `changes` job (dorny/paths-filter, with the same path lists the old workflows used) runs each PR oracle only when its own inputs change.
- The Chromium parser oracle keeps its push-to-main/davinci trigger and is skipped on PRs.
- A dispatch runs all four.
- `github-workflows-check-gate.test.ts` now pins this split.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `verify-tool-layout`, `v1-support-matrix --check`.
- `node --test` over workflow-layout, github-workflows*, content-mapper-_, lsp-dual-server-_, lsp-current-baseline*, lsp-vs-volar*, v1-_, official-_, release/_, rust-cache-docs-_, davinci-matrices and retired-tooling-languages: 1102 tests. The only failures are 2 cases in `release/verify-published-registry.test.ts`, and they fail the same way on main.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

<details><summary>I3 — merge proofs and integration workflows</summary>

#### Summary

Workflow cleanup, PR I3: merge the proof and integration workflows. 6 workflow files are folded into hosts.

| Workflow                | Action                                                             | Where the guard lives now                                                                                                                                                                                                                                           |
| ----------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| davinci-creusot         | Merged as job `creusot` of `davinci-kani` (the future `07-proofs`) | Same replay of the committed proofs (`cargo creusot --replay`). Its PR path list moves into the host's `changes` filter, and its push-to-main replay is kept. `davinci-creusot.test.ts` now checks the push paths and the `creusot` filter against the proof roots. |
| davinci-source-frames   | Merged as job `source-frames`                                      | Same Kani harness run and coverage witness check                                                                                                                                                                                                                    |
| rspack-src-block-ranges | Merged as job `source-ranges` of `rspack-hmr`                      | Same steps and drivers (`src-ranges-*`). Its paths were already a subset of the host's.                                                                                                                                                                             |
| vitest-module-cache     | Merged as job `module-cache` of `vitest-project-extends`           | Same cold, warm and source-change controls against the fixture's locked Vitest/Vue project                                                                                                                                                                          |
| viteplus-lint-consumers | Merged as job `lint-consumers` of `viteplus-config-consumers`      | Same tests, fed by the host's existing `full-source-cli` producer                                                                                                                                                                                                   |
| viteplus-task-cache     | Merged as job `task-cache`                                         | Same four-provider source-integration matrix. The published-version and external source-CLI-artifact dispatch inputs (and their inspect, download and extract steps) are dropped.                                                                                   |

`davinci-kani` now has a hosted `changes` job (dorny/paths-filter, using the old path lists). The Kani, Creusot and source-frame jobs each run only when their inputs change, so a Creusot-only PR does not start the Kani jobs. A dispatch runs all of them.

Not moved here: the proof-only jobs that #13608 leaves in `davinci-ir-graph`, `declaration-source-products`, `ir-pass-composition` and `style-module-products`. They can join this workflow in a follow-up once both PRs land.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `verify-tool-layout`, `vp fmt --check` on the changed files.
- `node --test` over workflow-layout, github-workflows*, davinci-_, viteplus-_, vitest-_, rspack_, rust-cache-docs-* and retired-tooling-languages: 778 tests. The workflow tests (workflow-layout, github-workflows*, davinci-creusot, davinci-reference-workflow) all pass. The 30 failures are in tests that need the source-built CLI or cargo locally (davinci-plugin-_, davinci-stage-_, viteplus-*-cli and others). They read none of the merged workflows, and the sampled ones fail the same way on main.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

<details><summary>K — move long inline commands into named programs</summary>

#### Summary

Part of the workflow cleanup (PR K, first batch). It adds a size guard for inline workflow code and moves 18 long commands into named programs.

**Guard.** `automationSizeViolations` (in `tools/support/ci/automation-commands.ts`) reports three things:

- a step `run` longer than 160 characters;
- a `node --test` with more than two inline test files;
- a job or step `if` longer than 200 characters.

The generator rejects any new violation. Existing ones are listed in `tools/support/ci/automation-size-legacy.ts`:

- Nothing may join the list.
- Entries leave as their commands move or their workflows are deleted.
- Entries of deleted workflows may linger so parallel deletion PRs do not conflict.

Protected release and trusted-publishing files are exempt; their commands stay as reviewed.

**Moved commands.** Each long command now lives in `.github/scripts/workflows/<workflow>/<step>.sh`, generated from Pkl and wrapped one argument group per line. The step calls `ci.workflowScript(...)`. Each driver body equals the old command once its line continuations are joined; a script compared all 18 against the previous YAML.

| Workflow                                      | Moved steps                                                |
| --------------------------------------------- | ---------------------------------------------------------- |
| editor-conformance                            | Verify the full host used this source-built binary         |
| jsr-package                                   | Test package generation                                    |
| nuxt2-bridge-module-build, nuxt2-module-build | Verify version-specific plans and original modern oracle   |
| nuxt3-module-build                            | 6 verification steps                                       |
| prettier-plugin                               | 2 tarball consumer steps                                   |
| release-smoke-contracts                       | 5 contract steps                                           |
| turbopack-consumers                           | Install consumer dependencies (was unnamed)                |
| typecheck-vs-vue-tsc                          | Build fixed normal product                                 |
| vite-precompile-cache                         | Verify in-memory, persistent and SSR precompile identities |
| volar-plugin                                  | Check public package declarations                          |

Not moved yet, and why:

- The core PR tiers (`pr-rust-checks`, `pr-rust-differential`, `pr-source-checks`, `full-tooling-plan`, `rust-pr-target-seed`) have custody and truth-table tests that pin their exact step texts against frozen fixtures.
- `davinci-incremental`, `davinci-reference` and `lint-reference-coverage-contracts` have tests that mutate the exact command strings.
- Steps in workflows that other open cleanup PRs are editing or deleting.
- `check.yml`, `pr-performance-observation`, `level-instruction-counts` and `shipped-profile-measure`, which are frozen during the PR drain.

All of these stay on the legacy list for a follow-up.

#### Verification

- `node tools/commands/generate/repository-config.ts --check`, `vp check`, `node tools/commands/ci/verify-tool-layout.ts`.
- `node --test tests/tooling/automation-commands.test.ts`: the new unit test covers all three rules.
- `node --test` over the 102 tooling tests that name the touched workflows. The only failures are cases that also fail on main locally: the GNU-tool cases in `setup-wild-http-recovery`, cargo-through-nix in `docs-stability`, and the registry cases.

#### Product impact

CI/tooling-only. No Rust, npm package or product source changes, so product binaries are unchanged.

</details>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
