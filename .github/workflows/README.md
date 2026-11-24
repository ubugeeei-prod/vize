# Workflow catalog

Edit the Pkl sources linked from each group. GitHub runs the generated YAML in this directory. Each step invokes a named program. Shell recipes live under `tools/config/pkl/scripts/` and workflow drivers under `.github/scripts/workflows/`. Jobs that qualify an older checkout stage the workflow revision’s drivers before switching source.
Run `node tools/commands/generate/repository-config.ts` to update the workflows and this catalog.

## Start here

Read the orchestration files first, then follow the reusable workflows they call.
Release and trusted-publishing files are listed for navigation, but remain protected
surfaces and are not routine CI-refactor targets.

| Step | File                                                 | Why read it                                           |
| ---- | ---------------------------------------------------- | ----------------------------------------------------- |
| 01   | [`check.yml`](./check.yml)                           | Top-level required-check fan-out and final gate       |
| 02   | [`pr-source-checks.yml`](./pr-source-checks.yml)     | Source-change planner and affected qualification      |
| 03   | [`pr-rust-checks.yml`](./pr-rust-checks.yml)         | Rust source checks and tooling shards                 |
| 04   | [`pr-rust-products.yml`](./pr-rust-products.yml)     | Affected Rust product matrix                          |
| 05   | [`pr-js-checks.yml`](./pr-js-checks.yml)             | Affected JavaScript package matrix                    |
| 06   | [`full-qualification.yml`](./full-qualification.yml) | Full-suite escalation path                            |
| 07   | [`tool-benchmark.yml`](./tool-benchmark.yml)         | Published cross-tool benchmark pipeline               |
| 08   | [`release.yml`](./release.yml)                       | Protected release orchestration; do not casually edit |

## Group index

| Group | Area                                                                                   | Workflows |
| ----- | -------------------------------------------------------------------------------------- | --------- |
| 01    | [Pull request orchestration](#01-pull-request-orchestration)                           | 20        |
| 02    | [Release and public acceptance](#02-release-and-public-acceptance)                     | 14        |
| 03    | [Benchmarks and profiling](#03-benchmarks-and-profiling)                               | 17        |
| 04    | [Type checking and content mapping](#04-type-checking-and-content-mapping)             | 5         |
| 05    | [Language services and editors](#05-language-services-and-editors)                     | 7         |
| 06    | [SFC, JSX, and compiler correctness](#06-sfc-jsx-and-compiler-correctness)             | 3         |
| 07    | [Davinci, Canon, and Level](#07-davinci-canon-and-level)                               | 11        |
| 08    | [Builders and framework integrations](#08-builders-and-framework-integrations)         | 13        |
| 09    | [Lint, format, and developer tooling](#09-lint-format-and-developer-tooling)           | 4         |
| 10    | [Applications and UI packages](#10-applications-and-ui-packages)                       | 2         |
| 11    | [Security, maintenance, and documentation](#11-security-maintenance-and-documentation) | 11        |

## 01 Pull request orchestration

Start here for required checks, change planning, merge queues, and full qualification.

[Edit the Pkl sources](../../tools/config/pkl/workflows/checks/)

| ID    | Workflow                          | File                                                                             |
| ----- | --------------------------------- | -------------------------------------------------------------------------------- |
| 01.01 | Rust feature tests                | [`01-rust-feature-tests.yml`](./01-rust-feature-tests.yml)                       |
| 01.02 | Canonical corpus divergence       | [`canonical-corpus-divergence.yml`](./canonical-corpus-divergence.yml)           |
| 01.03 | Canonical corpus objects          | [`canonical-corpus-objects.yml`](./canonical-corpus-objects.yml)                 |
| 01.04 | Benchmark - Type checking gate    | [`check-bench.yml`](./check-bench.yml)                                           |
| 01.05 | Check                             | [`check.yml`](./check.yml)                                                       |
| 01.06 | App E2E                           | [`e2e.yml`](./e2e.yml)                                                           |
| 01.07 | Full qualification                | [`full-qualification.yml`](./full-qualification.yml)                             |
| 01.08 | Full source CLI producer          | [`full-source-cli.yml`](./full-source-cli.yml)                                   |
| 01.09 | Full tooling plan                 | [`full-tooling-plan.yml`](./full-tooling-plan.yml)                               |
| 01.10 | PR JS package checks              | [`pr-js-checks.yml`](./pr-js-checks.yml)                                         |
| 01.11 | PR performance observation        | [`pr-performance-observation.yml`](./pr-performance-observation.yml)             |
| 01.12 | Rust source checks                | [`pr-rust-checks.yml`](./pr-rust-checks.yml)                                     |
| 01.13 | Required Rust differential checks | [`pr-rust-differential.yml`](./pr-rust-differential.yml)                         |
| 01.14 | Rust product checks               | [`pr-rust-products.yml`](./pr-rust-products.yml)                                 |
| 01.15 | PR source checks                  | [`pr-source-checks.yml`](./pr-source-checks.yml)                                 |
| 01.16 | Real Project Matrix differential  | [`real-project-matrix-differential.yml`](./real-project-matrix-differential.yml) |
| 01.17 | Real Project Matrix               | [`real-project-matrix.yml`](./real-project-matrix.yml)                           |
| 01.18 | Seed PR Rust target               | [`rust-pr-target-seed.yml`](./rust-pr-target-seed.yml)                           |
| 01.19 | Title Policy                      | [`title-policy.yml`](./title-policy.yml)                                         |
| 01.20 | V1 qualification manifest         | [`v1-supported-matrix.yml`](./v1-supported-matrix.yml)                           |

## 02 Release and public acceptance

Publication, registry, and installed-consumer evidence. Trusted-publishing workflows are protected surfaces.

[Edit the Pkl sources](../../tools/config/pkl/workflows/release/)

| ID    | Workflow                          | File                                                                 |
| ----- | --------------------------------- | -------------------------------------------------------------------- |
| 02.01 | JSR package checks                | [`jsr-package.yml`](./jsr-package.yml)                               |
| 02.02 | Nuxt and Vite release products    | [`nuxt-vite-release-products.yml`](./nuxt-vite-release-products.yml) |
| 02.03 | pkg.pr.new                        | [`pkg-pr-new.yml`](./pkg-pr-new.yml)                                 |
| 02.04 | Published fresh project           | [`published-fresh-project.yml`](./published-fresh-project.yml)       |
| 02.05 | Release build cache               | [`release-build-cache.yml`](./release-build-cache.yml)               |
| 02.06 | Release cadence                   | [`release-cadence.yml`](./release-cadence.yml)                       |
| 02.07 | Release contracts                 | [`release-contracts.yml`](./release-contracts.yml)                   |
| 02.08 | Release glibc floor               | [`release-glibc-floor.yml`](./release-glibc-floor.yml)               |
| 02.09 | Release JSR                       | [`release-jsr.yml`](./release-jsr.yml)                               |
| 02.10 | Prepare npm first-publish handoff | [`release-npm-bootstrap.yml`](./release-npm-bootstrap.yml)           |
| 02.11 | Release smoke contracts           | [`release-smoke-contracts.yml`](./release-smoke-contracts.yml)       |
| 02.12 | Release verify                    | [`release-verify.yml`](./release-verify.yml)                         |
| 02.13 | Release Zed host                  | [`release-zed-host.yml`](./release-zed-host.yml)                     |
| 02.14 | Release                           | [`release.yml`](./release.yml)                                       |

## 03 Benchmarks and profiling

Wall time, allocation, instruction, memory, scaling, and reproducible benchmark runs.

[Edit the Pkl sources](../../tools/config/pkl/workflows/performance/)

| ID    | Workflow                            | File                                                               |
| ----- | ----------------------------------- | ------------------------------------------------------------------ |
| 03.01 | Benchmark - Source comparison       | [`benchmark.yml`](./benchmark.yml)                                 |
| 03.02 | Benchmark - CI wall time            | [`ci-walltime-benchmark.yml`](./ci-walltime-benchmark.yml)         |
| 03.03 | Benchmark - SFC compilation         | [`compile-bench.yml`](./compile-bench.yml)                         |
| 03.04 | Content Mapper Cold Start           | [`content-mapper-cold-start.yml`](./content-mapper-cold-start.yml) |
| 03.05 | Benchmark - Criterion suites        | [`criterion-bench.yml`](./criterion-bench.yml)                     |
| 03.06 | Davinci Canon distinct-host scaling | [`davinci-canon-scaling.yml`](./davinci-canon-scaling.yml)         |
| 03.07 | Davinci resource budgets            | [`davinci-resource-budgets.yml`](./davinci-resource-budgets.yml)   |
| 03.08 | Benchmark - Fresco dimensions       | [`fresco-dimension-bench.yml`](./fresco-dimension-bench.yml)       |
| 03.09 | Level instruction counts            | [`level-instruction-counts.yml`](./level-instruction-counts.yml)   |
| 03.10 | Benchmark - Level memory            | [`level-memory-bench.yml`](./level-memory-bench.yml)               |
| 03.11 | Shipped release-profile measure     | [`shipped-profile-measure.yml`](./shipped-profile-measure.yml)     |
| 03.12 | Benchmark - Tool comparison         | [`tool-benchmark.yml`](./tool-benchmark.yml)                       |
| 03.13 | Type-check cold pair                | [`typecheck-cold-pair.yml`](./typecheck-cold-pair.yml)             |
| 03.14 | Type-check profile                  | [`typecheck-profile.yml`](./typecheck-profile.yml)                 |
| 03.15 | Type-check scaling                  | [`typecheck-scaling.yml`](./typecheck-scaling.yml)                 |
| 03.16 | Benchmark - VS Code startup         | [`vscode-startup-bench.yml`](./vscode-startup-bench.yml)           |
| 03.17 | Vue Benchmarks Replay               | [`vue-benchmarks-replay.yml`](./vue-benchmarks-replay.yml)         |

## 04 Type checking and content mapping

Typechecker, content-mapper, Corsa, and TypeScript-provider qualification.

[Edit the Pkl sources](../../tools/config/pkl/workflows/typecheck/)

| ID    | Workflow                           | File                                                                 |
| ----- | ---------------------------------- | -------------------------------------------------------------------- |
| 04.01 | Content Mapper Conformance         | [`content-mapper-conformance.yml`](./content-mapper-conformance.yml) |
| 04.02 | Typecheck vs vue-tsc               | [`typecheck-vs-vue-tsc.yml`](./typecheck-vs-vue-tsc.yml)             |
| 04.03 | WSL2 type-check qualification      | [`typecheck-wsl2.yml`](./typecheck-wsl2.yml)                         |
| 04.04 | Type checker native backend phases | [`typechecker-native-phases.yml`](./typechecker-native-phases.yml)   |
| 04.05 | Platform VSIX native projection    | [`typescript-provider-vsix.yml`](./typescript-provider-vsix.yml)     |

## 05 Language services and editors

LSP behavior, editor packages, VSIX hosts, overlays, navigation, and hover checks.

[Edit the Pkl sources](../../tools/config/pkl/workflows/editors/)

| ID    | Workflow                          | File                                                           |
| ----- | --------------------------------- | -------------------------------------------------------------- |
| 05.01 | Editor conformance                | [`editor-conformance.yml`](./editor-conformance.yml)           |
| 05.02 | Current-source LSP baseline       | [`lsp-current-baseline.yml`](./lsp-current-baseline.yml)       |
| 05.03 | LSP vs Vue language tools         | [`lsp-vs-volar.yml`](./lsp-vs-volar.yml)                       |
| 05.04 | TypeScript plugin packed consumer | [`typescript-plugin.yml`](./typescript-plugin.yml)             |
| 05.05 | Volar Plugin                      | [`volar-plugin.yml`](./volar-plugin.yml)                       |
| 05.06 | VS Code VSIX validation           | [`vscode-vsix-validation.yml`](./vscode-vsix-validation.yml)   |
| 05.07 | VS Code workspace server          | [`vscode-workspace-server.yml`](./vscode-workspace-server.yml) |

## 06 SFC, JSX, and compiler correctness

SFC compilation, JSX, bindings, templates, Vapor, recovery, and native correctness.

[Edit the Pkl sources](../../tools/config/pkl/workflows/compiler/)

| ID    | Workflow                   | File                                                                   |
| ----- | -------------------------- | ---------------------------------------------------------------------- |
| 06.01 | CSS engine forks           | [`css-engine-vendor-candidate.yml`](./css-engine-vendor-candidate.yml) |
| 06.02 | Native beta diagnostic DTO | [`native-beta-diagnostic-dto.yml`](./native-beta-diagnostic-dto.yml)   |
| 06.03 | Native Smoke               | [`native-smoke.yml`](./native-smoke.yml)                               |

## 07 Davinci, Canon, and Level

Davinci migration, Canon ownership, Level lowering, and cross-file graph contracts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/davinci/)

| ID    | Workflow                               | File                                                             |
| ----- | -------------------------------------- | ---------------------------------------------------------------- |
| 07.01 | Canon ecosystem producer qualification | [`canon-ecosystem-producer.yml`](./canon-ecosystem-producer.yml) |
| 07.02 | Canonical Davinci corpus               | [`davinci-canonical-corpus.yml`](./davinci-canonical-corpus.yml) |
| 07.03 | Davinci Incremental                    | [`davinci-incremental.yml`](./davinci-incremental.yml)           |
| 07.04 | Davinci IR graph queries               | [`davinci-ir-graph.yml`](./davinci-ir-graph.yml)                 |
| 07.05 | Davinci Kani                           | [`davinci-kani.yml`](./davinci-kani.yml)                         |
| 07.06 | Davinci Plugin SDK                     | [`davinci-plugin-sdk.yml`](./davinci-plugin-sdk.yml)             |
| 07.07 | Davinci reference backends             | [`davinci-reference.yml`](./davinci-reference.yml)               |
| 07.08 | IR pass composition                    | [`ir-pass-composition.yml`](./ir-pass-composition.yml)           |
| 07.09 | Level deletion readiness               | [`level-deletion-readiness.yml`](./level-deletion-readiness.yml) |
| 07.10 | Level dependencies                     | [`level-dependencies.yml`](./level-dependencies.yml)             |
| 07.11 | Level HTML content model               | [`level-html-content-model.yml`](./level-html-content-model.yml) |

## 08 Builders and framework integrations

Babel, Nuxt, Parcel, Rspack, Turbopack, Vite, Vitest, and consumer hosts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/integrations/)

| ID    | Workflow                               | File                                                               |
| ----- | -------------------------------------- | ------------------------------------------------------------------ |
| 08.01 | Babel plugin host contracts            | [`babel-plugin-hosts.yml`](./babel-plugin-hosts.yml)               |
| 08.02 | Nuxt scoped style build                | [`nuxt-style-build.yml`](./nuxt-style-build.yml)                   |
| 08.03 | Nuxt 2 Bridge host runtimes            | [`nuxt2-bridge-module-build.yml`](./nuxt2-bridge-module-build.yml) |
| 08.04 | Nuxt 2 module build                    | [`nuxt2-module-build.yml`](./nuxt2-module-build.yml)               |
| 08.05 | Nuxt 3 module build                    | [`nuxt3-module-build.yml`](./nuxt3-module-build.yml)               |
| 08.06 | Parcel transformer                     | [`parcel-transformer.yml`](./parcel-transformer.yml)               |
| 08.07 | Rspack SFC HMR                         | [`rspack-hmr.yml`](./rspack-hmr.yml)                               |
| 08.08 | Turbopack consumers                    | [`turbopack-consumers.yml`](./turbopack-consumers.yml)             |
| 08.09 | Vite+ Type Compatibility               | [`vite-plus-type-compat.yml`](./vite-plus-type-compat.yml)         |
| 08.10 | Vite precompile cache identities       | [`vite-precompile-cache.yml`](./vite-precompile-cache.yml)         |
| 08.11 | Vite+ configuration consumers          | [`viteplus-config-consumers.yml`](./viteplus-config-consumers.yml) |
| 08.12 | Vitest project configuration consumers | [`vitest-project-extends.yml`](./vitest-project-extends.yml)       |
| 08.13 | Void build                             | [`void-build.yml`](./void-build.yml)                               |

## 09 Lint, format, and developer tooling

ESLint, Oxlint, Prettier, formatter, doctor, configuration, and tooling contracts.

[Edit the Pkl sources](../../tools/config/pkl/workflows/tooling/)

| ID    | Workflow                          | File                                                                               |
| ----- | --------------------------------- | ---------------------------------------------------------------------------------- |
| 09.01 | ESLint plugin consumer            | [`eslint-plugin-consumer.yml`](./eslint-plugin-consumer.yml)                       |
| 09.02 | Lint reference coverage contracts | [`lint-reference-coverage-contracts.yml`](./lint-reference-coverage-contracts.yml) |
| 09.03 | n8n adoption fixture              | [`n8n-adoption.yml`](./n8n-adoption.yml)                                           |
| 09.04 | Prettier plugin consumer          | [`prettier-plugin.yml`](./prettier-plugin.yml)                                     |

## 10 Applications and UI packages

Musea, Fresco, playground, app E2E, UI packages, and browser-facing integrations.

[Edit the Pkl sources](../../tools/config/pkl/workflows/apps/)

| ID    | Workflow                        | File                                       |
| ----- | ------------------------------- | ------------------------------------------ |
| 10.01 | Fresco                          | [`fresco.yml`](./fresco.yml)               |
| 10.02 | Musea accessibility and capture | [`musea-browser.yml`](./musea-browser.yml) |

## 11 Security, maintenance, and documentation

Security audits, dependency upkeep, scheduled maintenance, docs, and repository hygiene.

[Edit the Pkl sources](../../tools/config/pkl/workflows/maintenance/)

| ID    | Workflow                      | File                                                                       |
| ----- | ----------------------------- | -------------------------------------------------------------------------- |
| 11.01 | Docs build                    | [`build-docs.yml`](./build-docs.yml)                                       |
| 11.02 | Clean obsolete merge groups   | [`cleanup-obsolete-merge-groups.yml`](./cleanup-obsolete-merge-groups.yml) |
| 11.03 | Declaration source products   | [`declaration-source-products.yml`](./declaration-source-products.yml)     |
| 11.04 | Deploy docs                   | [`deploy-docs.yml`](./deploy-docs.yml)                                     |
| 11.05 | Fuzz                          | [`fuzz.yml`](./fuzz.yml)                                                   |
| 11.06 | Language project fixture laws | [`language-project-fixtures.yml`](./language-project-fixtures.yml)         |
| 11.07 | Miri                          | [`miri.yml`](./miri.yml)                                                   |
| 11.08 | Sticky disk maintenance       | [`sticky-disk-maintenance.yml`](./sticky-disk-maintenance.yml)             |
| 11.09 | Style module products         | [`style-module-products.yml`](./style-module-products.yml)                 |
| 11.10 | Tool codegen equivalence      | [`tool-codegen-equivalence.yml`](./tool-codegen-equivalence.yml)           |
| 11.11 | Zizmor                        | [`zizmor.yml`](./zizmor.yml)                                               |
