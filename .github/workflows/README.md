<!-- DO NOT EDIT. Generated from tools/config/pkl/workflows. -->
<!-- Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`. -->

# Workflow catalog

Edit the Pkl sources linked from each group. GitHub runs the generated YAML in this directory. Each step invokes a named program. Shell recipes live under `tools/config/pkl/scripts/` and workflow drivers under `.github/scripts/workflows/`. Jobs that qualify an older checkout stage the workflow revision’s drivers before switching source.
Run `node tools/commands/generate/repository-config.ts` to update the workflows and this catalog.

## Start here

Read the orchestration files first, then follow the reusable workflows they call.
Release and trusted-publishing files are listed for navigation, but remain protected
surfaces and are not routine CI-refactor targets.

| Step | File                                                       | Why read it                                           |
| ---- | ---------------------------------------------------------- | ----------------------------------------------------- |
| 01   | [`01-check.yml`](./01-check.yml)                           | Top-level required-check fan-out and final gate       |
| 02   | [`01-pr-source.yml`](./01-pr-source.yml)                   | Source-change planner and affected qualification      |
| 03   | [`01-pr-rust.yml`](./01-pr-rust.yml)                       | Rust source checks and tooling shards                 |
| 04   | [`01-pr-rust-products.yml`](./01-pr-rust-products.yml)     | Affected Rust product matrix                          |
| 05   | [`01-pr-js.yml`](./01-pr-js.yml)                           | Affected JavaScript package matrix                    |
| 06   | [`01-full-qualification.yml`](./01-full-qualification.yml) | Full-suite escalation path                            |
| 07   | [`03-tool-comparison.yml`](./03-tool-comparison.yml)       | Published cross-tool benchmark pipeline               |
| 08   | [`release.yml`](./release.yml)                             | Protected release orchestration; do not casually edit |

## Group index

| Group | Area                                                                                   | Workflows |
| ----- | -------------------------------------------------------------------------------------- | --------- |
| 01    | [Pull request orchestration](#01-pull-request-orchestration)                           | 16        |
| 02    | [Release and public acceptance](#02-release-and-public-acceptance)                     | 13        |
| 03    | [Benchmarks and profiling](#03-benchmarks-and-profiling)                               | 19        |
| 04    | [Type checking and content mapping](#04-type-checking-and-content-mapping)             | 5         |
| 05    | [Language services and editors](#05-language-services-and-editors)                     | 8         |
| 06    | [SFC, JSX, and compiler correctness](#06-sfc-jsx-and-compiler-correctness)             | 4         |
| 07    | [Davinci, Canon, and Level](#07-davinci-canon-and-level)                               | 16        |
| 08    | [Builders and framework integrations](#08-builders-and-framework-integrations)         | 14        |
| 09    | [Lint, format, and developer tooling](#09-lint-format-and-developer-tooling)           | 4         |
| 10    | [Applications and UI packages](#10-applications-and-ui-packages)                       | 3         |
| 11    | [Security, maintenance, and documentation](#11-security-maintenance-and-documentation) | 7         |

## 01 Pull request orchestration

Start here for required checks, change planning, merge queues, and full qualification.

[Edit the Pkl sources](../../tools/config/pkl/workflows/checks/)

| ID    | Workflow                          | File                                                                     |
| ----- | --------------------------------- | ------------------------------------------------------------------------ |
| 01.01 | Check                             | [`01-check.yml`](./01-check.yml)                                         |
| 01.02 | Full qualification                | [`01-full-qualification.yml`](./01-full-qualification.yml)               |
| 01.03 | Full source CLI producer          | [`01-full-source-cli.yml`](./01-full-source-cli.yml)                     |
| 01.04 | Full tooling plan                 | [`01-full-tooling-plan.yml`](./01-full-tooling-plan.yml)                 |
| 01.05 | Clean obsolete merge groups       | [`01-merge-queue-cleanup.yml`](./01-merge-queue-cleanup.yml)             |
| 01.06 | PR JS package checks              | [`01-pr-js.yml`](./01-pr-js.yml)                                         |
| 01.07 | Required Rust differential checks | [`01-pr-rust-differential.yml`](./01-pr-rust-differential.yml)           |
| 01.08 | Rust product checks               | [`01-pr-rust-products.yml`](./01-pr-rust-products.yml)                   |
| 01.09 | Rust source checks                | [`01-pr-rust.yml`](./01-pr-rust.yml)                                     |
| 01.10 | PR source checks                  | [`01-pr-source.yml`](./01-pr-source.yml)                                 |
| 01.11 | Real Project Matrix differential  | [`01-real-project-differential.yml`](./01-real-project-differential.yml) |
| 01.12 | Real Project Matrix               | [`01-real-project-matrix.yml`](./01-real-project-matrix.yml)             |
| 01.13 | Rust feature tests                | [`01-rust-feature-tests.yml`](./01-rust-feature-tests.yml)               |
| 01.14 | Seed PR Rust target               | [`01-rust-target-seed.yml`](./01-rust-target-seed.yml)                   |
| 01.15 | Title Policy                      | [`01-title-policy.yml`](./01-title-policy.yml)                           |
| 01.16 | V1 qualification manifest         | [`01-v1-matrix.yml`](./01-v1-matrix.yml)                                 |

## 02 Release and public acceptance

Publication, registry, and installed-consumer evidence. Trusted-publishing workflows are protected surfaces.

[Edit the Pkl sources](../../tools/config/pkl/workflows/release/)

| ID    | Workflow                          | File                                                           |
| ----- | --------------------------------- | -------------------------------------------------------------- |
| 02.01 | JSR package checks                | [`jsr-package.yml`](./jsr-package.yml)                         |
| 02.02 | pkg.pr.new                        | [`pkg-pr-new.yml`](./pkg-pr-new.yml)                           |
| 02.03 | Published fresh project           | [`published-fresh-project.yml`](./published-fresh-project.yml) |
| 02.04 | Release build cache               | [`release-build-cache.yml`](./release-build-cache.yml)         |
| 02.05 | Release cadence                   | [`release-cadence.yml`](./release-cadence.yml)                 |
| 02.06 | Release contracts                 | [`release-contracts.yml`](./release-contracts.yml)             |
| 02.07 | Release glibc floor               | [`release-glibc-floor.yml`](./release-glibc-floor.yml)         |
| 02.08 | Release JSR                       | [`release-jsr.yml`](./release-jsr.yml)                         |
| 02.09 | Prepare npm first-publish handoff | [`release-npm-bootstrap.yml`](./release-npm-bootstrap.yml)     |
| 02.10 | Release smoke contracts           | [`release-smoke-contracts.yml`](./release-smoke-contracts.yml) |
| 02.11 | Release verify                    | [`release-verify.yml`](./release-verify.yml)                   |
| 02.12 | Release Zed host                  | [`release-zed-host.yml`](./release-zed-host.yml)               |
| 02.13 | Release                           | [`release.yml`](./release.yml)                                 |

## 03 Benchmarks and profiling

Wall time, allocation, instruction, memory, scaling, and reproducible benchmark runs.

[Edit the Pkl sources](../../tools/config/pkl/workflows/performance/)

| ID    | Workflow                            | File                                                           |
| ----- | ----------------------------------- | -------------------------------------------------------------- |
| 03.01 | Davinci Canon distinct-host scaling | [`03-canon-scaling.yml`](./03-canon-scaling.yml)               |
| 03.02 | Benchmark - Type checking gate      | [`03-check-gate.yml`](./03-check-gate.yml)                     |
| 03.03 | Benchmark - CI wall time            | [`03-ci-walltime.yml`](./03-ci-walltime.yml)                   |
| 03.04 | Benchmark - SFC compilation         | [`03-compile.yml`](./03-compile.yml)                           |
| 03.05 | Benchmark - Criterion suites        | [`03-criterion.yml`](./03-criterion.yml)                       |
| 03.06 | Benchmark - Fresco dimensions       | [`03-fresco-layout.yml`](./03-fresco-layout.yml)               |
| 03.07 | Level instruction counts            | [`03-instruction-counts.yml`](./03-instruction-counts.yml)     |
| 03.08 | Benchmark - Level memory            | [`03-level-memory.yml`](./03-level-memory.yml)                 |
| 03.09 | Davinci resource budgets            | [`03-lsp-resources.yml`](./03-lsp-resources.yml)               |
| 03.10 | PR performance observation          | [`03-pr-performance.yml`](./03-pr-performance.yml)             |
| 03.11 | Shipped release-profile measure     | [`03-release-profile.yml`](./03-release-profile.yml)           |
| 03.12 | Benchmark - Source comparison       | [`03-source-comparison.yml`](./03-source-comparison.yml)       |
| 03.13 | Benchmark - Tool comparison         | [`03-tool-comparison.yml`](./03-tool-comparison.yml)           |
| 03.14 | Type-check cold pair                | [`03-typecheck-compare.yml`](./03-typecheck-compare.yml)       |
| 03.15 | Type-check profile                  | [`03-typecheck-profile.yml`](./03-typecheck-profile.yml)       |
| 03.16 | Type-check scaling                  | [`03-typecheck-scaling.yml`](./03-typecheck-scaling.yml)       |
| 03.17 | Typecheck vs vue-tsc                | [`03-typecheck-vs-vue-tsc.yml`](./03-typecheck-vs-vue-tsc.yml) |
| 03.18 | Benchmark - VS Code startup         | [`03-vscode-startup.yml`](./03-vscode-startup.yml)             |
| 03.19 | Vue Benchmarks Replay               | [`03-vue-benchmarks.yml`](./03-vue-benchmarks.yml)             |

## 04 Type checking and content mapping

Typechecker, content-mapper, Corsa, and TypeScript-provider qualification.

[Edit the Pkl sources](../../tools/config/pkl/workflows/typecheck/)

| ID    | Workflow                           | File                                                                       |
| ----- | ---------------------------------- | -------------------------------------------------------------------------- |
| 04.01 | Content Mapper Cold Start          | [`04-content-mapper-cold-start.yml`](./04-content-mapper-cold-start.yml)   |
| 04.02 | Content Mapper Conformance         | [`04-content-mapper-conformance.yml`](./04-content-mapper-conformance.yml) |
| 04.03 | WSL2 type-check qualification      | [`04-typecheck-wsl2.yml`](./04-typecheck-wsl2.yml)                         |
| 04.04 | Type checker native backend phases | [`04-typechecker-native-phases.yml`](./04-typechecker-native-phases.yml)   |
| 04.05 | Platform VSIX native projection    | [`typescript-provider-vsix.yml`](./typescript-provider-vsix.yml)           |

## 05 Language services and editors

LSP behavior, editor packages, VSIX hosts, overlays, navigation, and hover checks.

[Edit the Pkl sources](../../tools/config/pkl/workflows/editors/)

| ID    | Workflow                          | File                                                                 |
| ----- | --------------------------------- | -------------------------------------------------------------------- |
| 05.01 | Editor conformance                | [`05-editor-conformance.yml`](./05-editor-conformance.yml)           |
| 05.02 | Current-source LSP baseline       | [`05-lsp-baseline.yml`](./05-lsp-baseline.yml)                       |
| 05.03 | LSP vs Vue language tools         | [`05-lsp-vs-volar.yml`](./05-lsp-vs-volar.yml)                       |
| 05.04 | Native beta diagnostic DTO        | [`05-native-diagnostic-dto.yml`](./05-native-diagnostic-dto.yml)     |
| 05.05 | Volar Plugin                      | [`05-volar-plugin.yml`](./05-volar-plugin.yml)                       |
| 05.06 | VS Code VSIX validation           | [`05-vscode-vsix.yml`](./05-vscode-vsix.yml)                         |
| 05.07 | VS Code workspace server          | [`05-vscode-workspace-server.yml`](./05-vscode-workspace-server.yml) |
| 05.08 | TypeScript plugin packed consumer | [`typescript-plugin.yml`](./typescript-plugin.yml)                   |

## 06 SFC, JSX, and compiler correctness

SFC compilation, JSX, bindings, templates, Vapor, recovery, and native correctness.

[Edit the Pkl sources](../../tools/config/pkl/workflows/compiler/)

| ID    | Workflow                 | File                                                         |
| ----- | ------------------------ | ------------------------------------------------------------ |
| 06.01 | Level HTML content model | [`06-browser-oracles.yml`](./06-browser-oracles.yml)         |
| 06.02 | Tool codegen equivalence | [`06-codegen-equivalence.yml`](./06-codegen-equivalence.yml) |
| 06.03 | CSS engine forks         | [`06-css-engine-forks.yml`](./06-css-engine-forks.yml)       |
| 06.04 | Native Smoke             | [`06-native-smoke.yml`](./06-native-smoke.yml)               |

## 07 Davinci, Canon, and Level

Davinci migration, Canon ownership, Level lowering, and cross-file graph contracts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/davinci/)

| ID    | Workflow                               | File                                                                         |
| ----- | -------------------------------------- | ---------------------------------------------------------------------------- |
| 07.01 | Canon ecosystem producer qualification | [`07-canon-ecosystem.yml`](./07-canon-ecosystem.yml)                         |
| 07.02 | Canonical corpus divergence            | [`07-canonical-corpus-divergence.yml`](./07-canonical-corpus-divergence.yml) |
| 07.03 | Canonical corpus objects               | [`07-canonical-corpus-objects.yml`](./07-canonical-corpus-objects.yml)       |
| 07.04 | Canonical Davinci corpus               | [`07-canonical-corpus.yml`](./07-canonical-corpus.yml)                       |
| 07.05 | Declaration source products            | [`07-declaration-products.yml`](./07-declaration-products.yml)               |
| 07.06 | Davinci Incremental                    | [`07-incremental.yml`](./07-incremental.yml)                                 |
| 07.07 | Davinci IR graph queries               | [`07-ir-graph.yml`](./07-ir-graph.yml)                                       |
| 07.08 | IR pass composition                    | [`07-ir-pass-composition.yml`](./07-ir-pass-composition.yml)                 |
| 07.09 | Level deletion readiness               | [`07-level-deletion-readiness.yml`](./07-level-deletion-readiness.yml)       |
| 07.10 | Level dependencies                     | [`07-level-dependencies.yml`](./07-level-dependencies.yml)                   |
| 07.11 | Davinci Plugin SDK                     | [`07-plugin-sdk.yml`](./07-plugin-sdk.yml)                                   |
| 07.12 | Davinci Creusot                        | [`07-proofs-creusot.yml`](./07-proofs-creusot.yml)                           |
| 07.13 | Davinci authored source frames         | [`07-proofs-source-frames.yml`](./07-proofs-source-frames.yml)               |
| 07.14 | Davinci Kani                           | [`07-proofs.yml`](./07-proofs.yml)                                           |
| 07.15 | Davinci reference backends             | [`07-reference-backends.yml`](./07-reference-backends.yml)                   |
| 07.16 | Style module products                  | [`07-style-module-products.yml`](./07-style-module-products.yml)             |

## 08 Builders and framework integrations

Babel, Nuxt, Parcel, Rspack, Turbopack, Vite, Vitest, and consumer hosts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/integrations/)

| ID    | Workflow                               | File                                                             |
| ----- | -------------------------------------- | ---------------------------------------------------------------- |
| 08.01 | Babel plugin host contracts            | [`08-babel.yml`](./08-babel.yml)                                 |
| 08.02 | Nuxt scoped style build                | [`08-nuxt-styles.yml`](./08-nuxt-styles.yml)                     |
| 08.03 | Nuxt and Vite release products         | [`08-nuxt-vite-products.yml`](./08-nuxt-vite-products.yml)       |
| 08.04 | Nuxt 2 Bridge host runtimes            | [`08-nuxt2-bridge.yml`](./08-nuxt2-bridge.yml)                   |
| 08.05 | Nuxt 2 module build                    | [`08-nuxt2-module.yml`](./08-nuxt2-module.yml)                   |
| 08.06 | Nuxt 3 module build                    | [`08-nuxt3-module.yml`](./08-nuxt3-module.yml)                   |
| 08.07 | Rspack SFC HMR                         | [`08-rspack.yml`](./08-rspack.yml)                               |
| 08.08 | Turbopack consumers                    | [`08-turbopack.yml`](./08-turbopack.yml)                         |
| 08.09 | Vite+ Type Compatibility               | [`08-vite-plus-types.yml`](./08-vite-plus-types.yml)             |
| 08.10 | Vite+ configuration consumers          | [`08-vite-plus.yml`](./08-vite-plus.yml)                         |
| 08.11 | Vite precompile cache identities       | [`08-vite-precompile-cache.yml`](./08-vite-precompile-cache.yml) |
| 08.12 | Vitest project configuration consumers | [`08-vitest.yml`](./08-vitest.yml)                               |
| 08.13 | Void build                             | [`08-void.yml`](./08-void.yml)                                   |
| 08.14 | Parcel transformer                     | [`parcel-transformer.yml`](./parcel-transformer.yml)             |

## 09 Lint, format, and developer tooling

ESLint, Oxlint, Prettier, formatter, doctor, configuration, and tooling contracts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/tooling/)

| ID    | Workflow                          | File                                                                 |
| ----- | --------------------------------- | -------------------------------------------------------------------- |
| 09.01 | ESLint plugin consumer            | [`09-eslint-plugin.yml`](./09-eslint-plugin.yml)                     |
| 09.02 | Lint reference coverage contracts | [`09-lint-reference-coverage.yml`](./09-lint-reference-coverage.yml) |
| 09.03 | n8n adoption fixture              | [`09-n8n-adoption.yml`](./09-n8n-adoption.yml)                       |
| 09.04 | Prettier plugin consumer          | [`09-prettier-plugin.yml`](./09-prettier-plugin.yml)                 |

## 10 Applications and UI packages

Musea, Fresco, playground, app E2E, UI packages, and browser-facing integrations.

[Edit the Pkl sources](../../tools/config/pkl/workflows/apps/)

| ID    | Workflow                        | File                                             |
| ----- | ------------------------------- | ------------------------------------------------ |
| 10.01 | App E2E                         | [`10-app-e2e.yml`](./10-app-e2e.yml)             |
| 10.02 | Fresco                          | [`10-fresco.yml`](./10-fresco.yml)               |
| 10.03 | Musea accessibility and capture | [`10-musea-browser.yml`](./10-musea-browser.yml) |

## 11 Security, maintenance, and documentation

Security audits, dependency upkeep, scheduled maintenance, docs, and repository hygiene.

[Edit the Pkl sources](../../tools/config/pkl/workflows/maintenance/)

| ID    | Workflow                      | File                                                               |
| ----- | ----------------------------- | ------------------------------------------------------------------ |
| 11.01 | Docs build                    | [`11-docs-build.yml`](./11-docs-build.yml)                         |
| 11.02 | Fuzz                          | [`11-fuzz.yml`](./11-fuzz.yml)                                     |
| 11.03 | Miri                          | [`11-miri.yml`](./11-miri.yml)                                     |
| 11.04 | Sticky disk maintenance       | [`11-sticky-disk.yml`](./11-sticky-disk.yml)                       |
| 11.05 | Zizmor                        | [`11-zizmor.yml`](./11-zizmor.yml)                                 |
| 11.06 | Deploy docs                   | [`deploy-docs.yml`](./deploy-docs.yml)                             |
| 11.07 | Language project fixture laws | [`language-project-fixtures.yml`](./language-project-fixtures.yml) |
