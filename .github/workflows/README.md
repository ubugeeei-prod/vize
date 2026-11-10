# Workflow catalog

GitHub only discovers workflow files directly under this directory, so the files
cannot be organized into subdirectories without changing how Actions finds them.
This generated, numbered catalog is the navigation layer; reusable implementation
belongs in `.github/actions/` or typed drivers under `tools/`.

Regenerate with `node tools/commands/ci/workflow-catalog.ts`.

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
| 02    | [Release and public acceptance](#02-release-and-public-acceptance)                     | 32        |
| 03    | [Benchmarks and profiling](#03-benchmarks-and-profiling)                               | 38        |
| 04    | [Type checking and content mapping](#04-type-checking-and-content-mapping)             | 15        |
| 05    | [Language services and editors](#05-language-services-and-editors)                     | 23        |
| 06    | [SFC, JSX, and compiler correctness](#06-sfc-jsx-and-compiler-correctness)             | 40        |
| 07    | [Davinci, Canon, and Level](#07-davinci-canon-and-level)                               | 21        |
| 08    | [Builders and framework integrations](#08-builders-and-framework-integrations)         | 23        |
| 09    | [Lint, format, and developer tooling](#09-lint-format-and-developer-tooling)           | 14        |
| 10    | [Applications and UI packages](#10-applications-and-ui-packages)                       | 4         |
| 11    | [Security, maintenance, and documentation](#11-security-maintenance-and-documentation) | 11        |

## 01 Pull request orchestration

Start here for required checks, change planning, merge queues, and full qualification.

| ID    | Workflow                           | File                                                                             |
| ----- | ---------------------------------- | -------------------------------------------------------------------------------- |
| 01.01 | Canonical corpus divergence        | [`canonical-corpus-divergence.yml`](./canonical-corpus-divergence.yml)           |
| 01.02 | Canonical corpus objects           | [`canonical-corpus-objects.yml`](./canonical-corpus-objects.yml)                 |
| 01.03 | Check Benchmark Gate               | [`check-bench.yml`](./check-bench.yml)                                           |
| 01.04 | Check                              | [`check.yml`](./check.yml)                                                       |
| 01.05 | App E2E                            | [`e2e.yml`](./e2e.yml)                                                           |
| 01.06 | Full qualification                 | [`full-qualification.yml`](./full-qualification.yml)                             |
| 01.07 | Full source CLI producer           | [`full-source-cli.yml`](./full-source-cli.yml)                                   |
| 01.08 | Full tooling plan                  | [`full-tooling-plan.yml`](./full-tooling-plan.yml)                               |
| 01.09 | Full worker cancellation contracts | [`full-worker-contracts.yml`](./full-worker-contracts.yml)                       |
| 01.10 | PR JS package checks               | [`pr-js-checks.yml`](./pr-js-checks.yml)                                         |
| 01.11 | PR performance observation         | [`pr-performance-observation.yml`](./pr-performance-observation.yml)             |
| 01.12 | Rust source checks                 | [`pr-rust-checks.yml`](./pr-rust-checks.yml)                                     |
| 01.13 | Required Rust differential checks  | [`pr-rust-differential.yml`](./pr-rust-differential.yml)                         |
| 01.14 | Rust product checks                | [`pr-rust-products.yml`](./pr-rust-products.yml)                                 |
| 01.15 | PR source checks                   | [`pr-source-checks.yml`](./pr-source-checks.yml)                                 |
| 01.16 | PR source CLI producer             | [`pr-source-cli.yml`](./pr-source-cli.yml)                                       |
| 01.17 | Real Project Matrix differential   | [`real-project-matrix-differential.yml`](./real-project-matrix-differential.yml) |
| 01.18 | Real Project Matrix                | [`real-project-matrix.yml`](./real-project-matrix.yml)                           |
| 01.19 | Seed PR Rust target                | [`rust-pr-target-seed.yml`](./rust-pr-target-seed.yml)                           |
| 01.20 | Title Policy                       | [`title-policy.yml`](./title-policy.yml)                                         |

## 02 Release and public acceptance

Publication, registry, and installed-consumer evidence. Trusted-publishing workflows are protected surfaces.

| ID    | Workflow                                  | File                                                                                       |
| ----- | ----------------------------------------- | ------------------------------------------------------------------------------------------ |
| 02.01 | Corsa package refresh                     | [`corsa-package-refresh.yml`](./corsa-package-refresh.yml)                                 |
| 02.02 | Public editor installation custody        | [`editor-vite-stub-public.yml`](./editor-vite-stub-public.yml)                             |
| 02.03 | Empty generic public contract             | [`empty-generic-public-contract.yml`](./empty-generic-public-contract.yml)                 |
| 02.04 | Public formatter profile acceptance       | [`formatter-profiles-public.yml`](./formatter-profiles-public.yml)                         |
| 02.05 | JSR package checks                        | [`jsr-package.yml`](./jsr-package.yml)                                                     |
| 02.06 | Public JSR types and runtime capabilities | [`jsr-public-types.yml`](./jsr-public-types.yml)                                           |
| 02.07 | Shipped Linux LSP resource comparison     | [`lsp-release-resources.yml`](./lsp-release-resources.yml)                                 |
| 02.08 | Nuxt public runtime dependencies          | [`nuxt-public-dependencies.yml`](./nuxt-public-dependencies.yml)                           |
| 02.09 | Public attribute ignore acceptance        | [`patina-attribute-ignore-public.yml`](./patina-attribute-ignore-public.yml)               |
| 02.10 | Public strict boolean runtime acceptance  | [`patina-boolean-public.yml`](./patina-boolean-public.yml)                                 |
| 02.11 | pkg.pr.new                                | [`pkg-pr-new.yml`](./pkg-pr-new.yml)                                                       |
| 02.12 | Published plain-script import acceptance  | [`plain-script-import-public-acceptance.yml`](./plain-script-import-public-acceptance.yml) |
| 02.13 | Published JSX child values                | [`public-jsx-child-values.yml`](./public-jsx-child-values.yml)                             |
| 02.14 | Public native event casing                | [`public-native-events.yml`](./public-native-events.yml)                                   |
| 02.15 | Public typechecker consumer               | [`public-typechecker-consumer.yml`](./public-typechecker-consumer.yml)                     |
| 02.16 | Published original ambient template ref   | [`published-ambient-template-ref.yml`](./published-ambient-template-ref.yml)               |
| 02.17 | Published component ARIA                  | [`published-component-aria.yml`](./published-component-aria.yml)                           |
| 02.18 | Published computed event names            | [`published-computed-event-names.yml`](./published-computed-event-names.yml)               |
| 02.19 | Published original constructor 27         | [`published-constructor-original27.yml`](./published-constructor-original27.yml)           |
| 02.20 | Published human reports 11252 and 11253   | [`published-human-reports-11252-11253.yml`](./published-human-reports-11252-11253.yml)     |
| 02.21 | Published slot instance NoAny             | [`published-slot-instance-noany.yml`](./published-slot-instance-noany.yml)                 |
| 02.22 | Published slot provider controls          | [`published-slot-provider.yml`](./published-slot-provider.yml)                             |
| 02.23 | Queue release gates                       | [`queue-release-gates.yml`](./queue-release-gates.yml)                                     |
| 02.24 | Release build cache                       | [`release-build-cache.yml`](./release-build-cache.yml)                                     |
| 02.25 | Release cadence                           | [`release-cadence.yml`](./release-cadence.yml)                                             |
| 02.26 | Release contracts                         | [`release-contracts.yml`](./release-contracts.yml)                                         |
| 02.27 | Release JSR                               | [`release-jsr.yml`](./release-jsr.yml)                                                     |
| 02.28 | Release smoke contracts                   | [`release-smoke-contracts.yml`](./release-smoke-contracts.yml)                             |
| 02.29 | Release verify                            | [`release-verify.yml`](./release-verify.yml)                                               |
| 02.30 | Release Zed host                          | [`release-zed-host.yml`](./release-zed-host.yml)                                           |
| 02.31 | Release                                   | [`release.yml`](./release.yml)                                                             |
| 02.32 | Published TypeScript provider acceptance  | [`typescript-provider-public.yml`](./typescript-provider-public.yml)                       |

## 03 Benchmarks and profiling

Wall time, allocation, instruction, memory, scaling, and reproducible benchmark runs.

| ID    | Workflow                                            | File                                                                                     |
| ----- | --------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| 03.01 | Benchmark                                           | [`benchmark.yml`](./benchmark.yml)                                                       |
| 03.02 | CI walltime comparison                              | [`ci-walltime-benchmark.yml`](./ci-walltime-benchmark.yml)                               |
| 03.03 | Cold typechecker scale                              | [`cold-typechecker-scale.yml`](./cold-typechecker-scale.yml)                             |
| 03.04 | Compile Benchmark                                   | [`compile-bench.yml`](./compile-bench.yml)                                               |
| 03.05 | Component binding view allocation profile           | [`component-binding-view-profile.yml`](./component-binding-view-profile.yml)             |
| 03.06 | Component scope binding allocation profile          | [`component-scope-binding.yml`](./component-scope-binding.yml)                           |
| 03.07 | Content Mapper Cold Start                           | [`content-mapper-cold-start.yml`](./content-mapper-cold-start.yml)                       |
| 03.08 | Content mapper span allocation                      | [`content-mapper-span-allocation.yml`](./content-mapper-span-allocation.yml)             |
| 03.09 | Criterion Bench                                     | [`criterion-bench.yml`](./criterion-bench.yml)                                           |
| 03.10 | Davinci Canon distinct-host scaling                 | [`davinci-canon-scaling.yml`](./davinci-canon-scaling.yml)                               |
| 03.11 | Davinci resource budgets                            | [`davinci-resource-budgets.yml`](./davinci-resource-budgets.yml)                         |
| 03.12 | Doctor filter benchmark                             | [`doctor-filter-bench.yml`](./doctor-filter-bench.yml)                                   |
| 03.13 | Document snapshot source profile                    | [`document-snapshot-source-profile.yml`](./document-snapshot-source-profile.yml)         |
| 03.14 | Fresco dimension benchmark                          | [`fresco-dimension-bench.yml`](./fresco-dimension-bench.yml)                             |
| 03.15 | Expression slow unit 10401 profile                  | [`fuzz-expression-10401-profile.yml`](./fuzz-expression-10401-profile.yml)               |
| 03.16 | Level instruction counts                            | [`level-instruction-counts.yml`](./level-instruction-counts.yml)                         |
| 03.17 | Complete Level Memory Comparison                    | [`level-memory-bench.yml`](./level-memory-bench.yml)                                     |
| 03.18 | Same-wire LSP benchmark                             | [`lsp-dual-server-benchmark.yml`](./lsp-dual-server-benchmark.yml)                       |
| 03.19 | LSP marker scan allocation and residency comparison | [`lsp-marker-scan-scale.yml`](./lsp-marker-scan-scale.yml)                               |
| 03.20 | LSP source operations and allocations               | [`lsp-source-profile.yml`](./lsp-source-profile.yml)                                     |
| 03.21 | Partition weight work                               | [`partition-weight-work.yml`](./partition-weight-work.yml)                               |
| 03.22 | Persistent batch storage                            | [`persistent-batch-storage.yml`](./persistent-batch-storage.yml)                         |
| 03.23 | Profile allocation thread law                       | [`profile-allocation-thread-law.yml`](./profile-allocation-thread-law.yml)               |
| 03.24 | Script recovery clone cost                          | [`script-recovery-clone-cost.yml`](./script-recovery-clone-cost.yml)                     |
| 03.25 | Script recovery cost                                | [`script-recovery-cost.yml`](./script-recovery-cost.yml)                                 |
| 03.26 | Setup binding anchor allocation profile             | [`setup-binding-anchor-profile.yml`](./setup-binding-anchor-profile.yml)                 |
| 03.27 | Native SFC member phase comparison                  | [`sfc-member-profile.yml`](./sfc-member-profile.yml)                                     |
| 03.28 | Source operation profile                            | [`source-operation-profile.yml`](./source-operation-profile.yml)                         |
| 03.29 | String Storage Bench                                | [`string-storage-bench.yml`](./string-storage-bench.yml)                                 |
| 03.30 | Text boundary benchmark                             | [`text-boundary-bench.yml`](./text-boundary-bench.yml)                                   |
| 03.31 | Tool Benchmark                                      | [`tool-benchmark.yml`](./tool-benchmark.yml)                                             |
| 03.32 | Type admission scale axes                           | [`type-admission-scale.yml`](./type-admission-scale.yml)                                 |
| 03.33 | Type source snapshot benchmark                      | [`type-snapshot-bench.yml`](./type-snapshot-bench.yml)                                   |
| 03.34 | Type-check cold pair                                | [`typecheck-cold-pair.yml`](./typecheck-cold-pair.yml)                                   |
| 03.35 | Type-check scaling                                  | [`typecheck-scaling.yml`](./typecheck-scaling.yml)                                       |
| 03.36 | VS Code startup bench                               | [`vscode-startup-bench.yml`](./vscode-startup-bench.yml)                                 |
| 03.37 | Vue Benchmarks Current Qualification                | [`vue-benchmarks-current-qualification.yml`](./vue-benchmarks-current-qualification.yml) |
| 03.38 | Vue Benchmarks Replay                               | [`vue-benchmarks-replay.yml`](./vue-benchmarks-replay.yml)                               |

## 04 Type checking and content mapping

Typechecker, content-mapper, Corsa, and TypeScript-provider qualification.

| ID    | Workflow                              | File                                                                                         |
| ----- | ------------------------------------- | -------------------------------------------------------------------------------------------- |
| 04.01 | Checker constructor emitter parity    | [`checker-constructor-emitter.yml`](./checker-constructor-emitter.yml)                       |
| 04.02 | Component input diagnostic ownership  | [`component-input-diagnostic-ownership.yml`](./component-input-diagnostic-ownership.yml)     |
| 04.03 | Component input projection packets    | [`component-input-projection-packets.yml`](./component-input-projection-packets.yml)         |
| 04.04 | Content Mapper Conformance            | [`content-mapper-conformance.yml`](./content-mapper-conformance.yml)                         |
| 04.05 | Consumer instance globals capture     | [`content-mapper-instance-globals.yml`](./content-mapper-instance-globals.yml)               |
| 04.06 | Content mapper pair                   | [`content-mapper-pair.yml`](./content-mapper-pair.yml)                                       |
| 04.07 | Content Mapper Source Case Jobs       | [`content-mapper-reported-q16.yml`](./content-mapper-reported-q16.yml)                       |
| 04.08 | Content Mapper worker stack           | [`content-mapper-worker-stack.yml`](./content-mapper-worker-stack.yml)                       |
| 04.09 | Type-check corpus pair                | [`typecheck-corpus-pair.yml`](./typecheck-corpus-pair.yml)                                   |
| 04.10 | Typecheck vs vue-tsc                  | [`typecheck-vs-vue-tsc.yml`](./typecheck-vs-vue-tsc.yml)                                     |
| 04.11 | WSL2 type-check qualification         | [`typecheck-wsl2.yml`](./typecheck-wsl2.yml)                                                 |
| 04.12 | Typechecker cache custody             | [`typechecker-cache-custody.yml`](./typechecker-cache-custody.yml)                           |
| 04.13 | Type checker native backend phases    | [`typechecker-native-phases.yml`](./typechecker-native-phases.yml)                           |
| 04.14 | Native syntax and semantic completion | [`typechecker-syntax-semantic-completion.yml`](./typechecker-syntax-semantic-completion.yml) |
| 04.15 | Platform VSIX native projection       | [`typescript-provider-vsix.yml`](./typescript-provider-vsix.yml)                             |

## 05 Language services and editors

LSP behavior, editor packages, VSIX hosts, overlays, navigation, and hover checks.

| ID    | Workflow                                  | File                                                                         |
| ----- | ----------------------------------------- | ---------------------------------------------------------------------------- |
| 05.01 | Packaged CSS hover visual evidence        | [`css-hover-visual.yml`](./css-hover-visual.yml)                             |
| 05.02 | Editor conformance                        | [`editor-conformance.yml`](./editor-conformance.yml)                         |
| 05.03 | LSP config startup diagnostics            | [`lsp-config-diagnostics.yml`](./lsp-config-diagnostics.yml)                 |
| 05.04 | Current-source LSP baseline               | [`lsp-current-baseline.yml`](./lsp-current-baseline.yml)                     |
| 05.05 | Focused real-project LSP qualification    | [`lsp-focused-projects.yml`](./lsp-focused-projects.yml)                     |
| 05.06 | JSX intrinsic hover observations          | [`lsp-jsx-intrinsic-hover.yml`](./lsp-jsx-intrinsic-hover.yml)               |
| 05.07 | LSP native diagnostic coverage            | [`lsp-native-diagnostic-coverage.yml`](./lsp-native-diagnostic-coverage.yml) |
| 05.08 | Native diagnostic warm-up                 | [`lsp-native-diagnostic-warmup.yml`](./lsp-native-diagnostic-warmup.yml)     |
| 05.09 | Native LSP overlay custody                | [`lsp-overlay-custody.yml`](./lsp-overlay-custody.yml)                       |
| 05.10 | LSP independent overlay graphs            | [`lsp-overlay-graph-reuse.yml`](./lsp-overlay-graph-reuse.yml)               |
| 05.11 | LSP exact-parent residency pair           | [`lsp-residency-pair.yml`](./lsp-residency-pair.yml)                         |
| 05.12 | LSP indexed scheduler source cost carrier | [`lsp-scheduler-cost-carrier.yml`](./lsp-scheduler-cost-carrier.yml)         |
| 05.13 | Tag prefix classification observations    | [`lsp-tag-prefix-classification.yml`](./lsp-tag-prefix-classification.yml)   |
| 05.14 | LSP vs Vue language tools                 | [`lsp-vs-volar.yml`](./lsp-vs-volar.yml)                                     |
| 05.15 | Native named SFC value hover              | [`named-sfc-value-hover.yml`](./named-sfc-value-hover.yml)                   |
| 05.16 | Router installed editor artifact          | [`router-editor-artifact.yml`](./router-editor-artifact.yml)                 |
| 05.17 | Published router navigation matrix        | [`router-experimental-navigation.yml`](./router-experimental-navigation.yml) |
| 05.18 | Router preview source provider            | [`router-preview-conformance.yml`](./router-preview-conformance.yml)         |
| 05.19 | TypeScript plugin packed consumer         | [`typescript-plugin.yml`](./typescript-plugin.yml)                           |
| 05.20 | Volar Plugin                              | [`volar-plugin.yml`](./volar-plugin.yml)                                     |
| 05.21 | VS Code VSIX validation                   | [`vscode-vsix-validation.yml`](./vscode-vsix-validation.yml)                 |
| 05.22 | VS Code workspace server                  | [`vscode-workspace-server.yml`](./vscode-workspace-server.yml)               |
| 05.23 | Vue 2 editor diagnostic context           | [`vue2-editor-diagnostic-context.yml`](./vue2-editor-diagnostic-context.yml) |

## 06 SFC, JSX, and compiler correctness

SFC compilation, JSX, bindings, templates, Vapor, recovery, and native correctness.

| ID    | Workflow                                 | File                                                                           |
| ----- | ---------------------------------------- | ------------------------------------------------------------------------------ |
| 06.01 | Ambient template ref ownership           | [`ambient-template-ref-ownership.yml`](./ambient-template-ref-ownership.yml)   |
| 06.02 | SFC asset import bindings                | [`asset-import-bindings.yml`](./asset-import-bindings.yml)                     |
| 06.03 | Async component render ancestry          | [`async-component-ancestry.yml`](./async-component-ancestry.yml)               |
| 06.04 | Binding origin source laws               | [`binding-origin-laws.yml`](./binding-origin-laws.yml)                         |
| 06.05 | Compiler options snapshot                | [`compiler-options-snapshot.yml`](./compiler-options-snapshot.yml)             |
| 06.06 | Cross-file script coordinate packets     | [`cross-file-script-offset.yml`](./cross-file-script-offset.yml)               |
| 06.07 | CSS color declaration admission          | [`css-color-declaration-admission.yml`](./css-color-declaration-admission.yml) |
| 06.08 | CSS color literal hover                  | [`css-color-literal-hover.yml`](./css-color-literal-hover.yml)                 |
| 06.09 | Native CSS source maps                   | [`css-compile-source-maps.yml`](./css-compile-source-maps.yml)                 |
| 06.10 | CSS engine forks                         | [`css-engine-vendor-candidate.yml`](./css-engine-vendor-candidate.yml)         |
| 06.11 | Optional inject fallback contracts       | [`defaulted-inject.yml`](./defaulted-inject.yml)                               |
| 06.12 | Expression overlapping speculation audit | [`expression-overlap-audit.yml`](./expression-overlap-audit.yml)               |
| 06.13 | Decoded loop bindings                    | [`for-bindings.yml`](./for-bindings.yml)                                       |
| 06.14 | JSX authored native modules              | [`jsx-authored-native-modules.yml`](./jsx-authored-native-modules.yml)         |
| 06.15 | JSX backend source origins               | [`jsx-backend-origins.yml`](./jsx-backend-origins.yml)                         |
| 06.16 | L1 document special text                 | [`l1-document-special-text.yml`](./l1-document-special-text.yml)               |
| 06.17 | Narrowed slot host guard                 | [`narrowed-slot-host-guard.yml`](./narrowed-slot-host-guard.yml)               |
| 06.18 | Native beta diagnostic DTO               | [`native-beta-diagnostic-dto.yml`](./native-beta-diagnostic-dto.yml)           |
| 06.19 | Native release token layout              | [`native-build-token-layout.yml`](./native-build-token-layout.yml)             |
| 06.20 | Native checker CPU attribution           | [`native-checker-cpu-metadata.yml`](./native-checker-cpu-metadata.yml)         |
| 06.21 | Native image-alt query source evidence   | [`native-image-alt-query.yml`](./native-image-alt-query.yml)                   |
| 06.22 | Native operational incident trace        | [`native-ops-incident.yml`](./native-ops-incident.yml)                         |
| 06.23 | SFC runtime option transport             | [`native-sfc-runtime-options.yml`](./native-sfc-runtime-options.yml)           |
| 06.24 | SFC runtime version selection            | [`native-sfc-runtime-version.yml`](./native-sfc-runtime-version.yml)           |
| 06.25 | Native Smoke                             | [`native-smoke.yml`](./native-smoke.yml)                                       |
| 06.26 | Leading newline runtime                  | [`pre-textarea-first-newline.yml`](./pre-textarea-first-newline.yml)           |
| 06.27 | Provide type assertion whole packets     | [`provide-type-assertion.yml`](./provide-type-assertion.yml)                   |
| 06.28 | Recovery normal codegen                  | [`recovery-normal-codegen.yml`](./recovery-normal-codegen.yml)                 |
| 06.29 | Scoped nesting specificity               | [`scoped-nesting-specificity.yml`](./scoped-nesting-specificity.yml)           |
| 06.30 | Current native SFC lifecycle             | [`sfc-optional-lifecycle.yml`](./sfc-optional-lifecycle.yml)                   |
| 06.31 | Native parsed SFC attributes             | [`sfc-parse-attributes.yml`](./sfc-parse-attributes.yml)                       |
| 06.32 | Historical SFC Parse Replay              | [`sfc-parse-replay.yml`](./sfc-parse-replay.yml)                               |
| 06.33 | Retained source-line replay              | [`source-symbolizer-retained.yml`](./source-symbolizer-retained.yml)           |
| 06.34 | Source symbolizer smoke                  | [`source-symbolizer-smoke.yml`](./source-symbolizer-smoke.yml)                 |
| 06.35 | Public mapper comparison                 | [`template-alias-paired-11822.yml`](./template-alias-paired-11822.yml)         |
| 06.36 | Template compiler global policy          | [`template-global-policy.yml`](./template-global-policy.yml)                   |
| 06.37 | Typed context undefined contracts        | [`typed-context-undefined.yml`](./typed-context-undefined.yml)                 |
| 06.38 | Unicode prop projection                  | [`unicode-prop-projection.yml`](./unicode-prop-projection.yml)                 |
| 06.39 | Vapor literal object styles              | [`vapor-literal-object-styles.yml`](./vapor-literal-object-styles.yml)         |
| 06.40 | Native Vue 2 IR template target          | [`vue2-ir-template.yml`](./vue2-ir-template.yml)                               |

## 07 Davinci, Canon, and Level

Davinci migration, Canon ownership, Level lowering, and cross-file graph contracts.

| ID    | Workflow                                | File                                                                           |
| ----- | --------------------------------------- | ------------------------------------------------------------------------------ |
| 07.01 | Canon component casing accuracy         | [`canon-component-casing.yml`](./canon-component-casing.yml)                   |
| 07.02 | Canon helper native baseline            | [`canon-helper-native-baseline.yml`](./canon-helper-native-baseline.yml)       |
| 07.03 | Canon instance directive ownership      | [`canon-instance-directives.yml`](./canon-instance-directives.yml)             |
| 07.04 | Canon slot constructor admission        | [`canon-slot-constructor.yml`](./canon-slot-constructor.yml)                   |
| 07.05 | Canon slot provider ownership           | [`canon-slot-provider-any.yml`](./canon-slot-provider-any.yml)                 |
| 07.06 | Compact Collections                     | [`compact-collections.yml`](./compact-collections.yml)                         |
| 07.07 | Cross-file finalization work comparison | [`cross-file-finalization-compare.yml`](./cross-file-finalization-compare.yml) |
| 07.08 | Cross-file finalization order           | [`cross-file-finalization-order.yml`](./cross-file-finalization-order.yml)     |
| 07.09 | Cross-file dependency profiling         | [`cross-file-finalization.yml`](./cross-file-finalization.yml)                 |
| 07.10 | Cross-file SFC effect graphs            | [`cross-file-sfc-effects.yml`](./cross-file-sfc-effects.yml)                   |
| 07.11 | Cross-file retained import updates      | [`cross-file-source-updates.yml`](./cross-file-source-updates.yml)             |
| 07.12 | Cross-file WASM effect grammars         | [`cross-file-wasm-effects.yml`](./cross-file-wasm-effects.yml)                 |
| 07.13 | Canonical Davinci corpus                | [`davinci-canonical-corpus.yml`](./davinci-canonical-corpus.yml)               |
| 07.14 | Davinci Incremental                     | [`davinci-incremental.yml`](./davinci-incremental.yml)                         |
| 07.15 | Davinci Kani                            | [`davinci-kani.yml`](./davinci-kani.yml)                                       |
| 07.16 | Davinci Plugin SDK                      | [`davinci-plugin-sdk.yml`](./davinci-plugin-sdk.yml)                           |
| 07.17 | Davinci reference backends              | [`davinci-reference.yml`](./davinci-reference.yml)                             |
| 07.18 | Davinci unused bindings                 | [`davinci-unused-bindings.yml`](./davinci-unused-bindings.yml)                 |
| 07.19 | Level deletion readiness                | [`level-deletion-readiness.yml`](./level-deletion-readiness.yml)               |
| 07.20 | Level dependencies                      | [`level-dependencies.yml`](./level-dependencies.yml)                           |
| 07.21 | Level HTML content model                | [`level-html-content-model.yml`](./level-html-content-model.yml)               |

## 08 Builders and framework integrations

Babel, Nuxt, Parcel, Rspack, Turbopack, Vite, Vitest, and consumer hosts.

| ID    | Workflow                               | File                                                                       |
| ----- | -------------------------------------- | -------------------------------------------------------------------------- |
| 08.01 | Babel plugin host contracts            | [`babel-plugin-hosts.yml`](./babel-plugin-hosts.yml)                       |
| 08.02 | Babel rc9 ABI frontier                 | [`babel-rc9-frontier.yml`](./babel-rc9-frontier.yml)                       |
| 08.03 | JS install dependency closure          | [`js-install-closure.yml`](./js-install-closure.yml)                       |
| 08.04 | Nuxt generated declaration membership  | [`nuxt-generated-membership.yml`](./nuxt-generated-membership.yml)         |
| 08.05 | Nuxt scoped style build                | [`nuxt-style-build.yml`](./nuxt-style-build.yml)                           |
| 08.06 | Nuxt 2 Bridge host runtimes            | [`nuxt2-bridge-module-build.yml`](./nuxt2-bridge-module-build.yml)         |
| 08.07 | Nuxt 2 module build                    | [`nuxt2-module-build.yml`](./nuxt2-module-build.yml)                       |
| 08.08 | Nuxt 3 module build                    | [`nuxt3-module-build.yml`](./nuxt3-module-build.yml)                       |
| 08.09 | Parcel transformer                     | [`parcel-transformer.yml`](./parcel-transformer.yml)                       |
| 08.10 | Plain-script import ref runtime        | [`plain-script-import-ref.yml`](./plain-script-import-ref.yml)             |
| 08.11 | Rspack SFC HMR                         | [`rspack-hmr.yml`](./rspack-hmr.yml)                                       |
| 08.12 | Rspack external block ranges           | [`rspack-src-block-ranges.yml`](./rspack-src-block-ranges.yml)             |
| 08.13 | Turbopack consumers                    | [`turbopack-consumers.yml`](./turbopack-consumers.yml)                     |
| 08.14 | Vite+ Type Compatibility               | [`vite-plus-type-compat.yml`](./vite-plus-type-compat.yml)                 |
| 08.15 | Vite precompile cache identities       | [`vite-precompile-cache.yml`](./vite-precompile-cache.yml)                 |
| 08.16 | Vite relative style origins            | [`vite-style-relative-imports.yml`](./vite-style-relative-imports.yml)     |
| 08.17 | Vite+ configuration consumers          | [`viteplus-config-consumers.yml`](./viteplus-config-consumers.yml)         |
| 08.18 | Vite+ lint configuration consumers     | [`viteplus-lint-consumers.yml`](./viteplus-lint-consumers.yml)             |
| 08.19 | Vite+ task cache consumers             | [`viteplus-task-cache.yml`](./viteplus-task-cache.yml)                     |
| 08.20 | Vitest persistent Vue modules          | [`vitest-module-cache.yml`](./vitest-module-cache.yml)                     |
| 08.21 | Vitest project configuration consumers | [`vitest-project-extends.yml`](./vitest-project-extends.yml)               |
| 08.22 | Void build                             | [`void-build.yml`](./void-build.yml)                                       |
| 08.23 | VueFes SSR style dependencies          | [`vuefes-ssr-style-dependencies.yml`](./vuefes-ssr-style-dependencies.yml) |

## 09 Lint, format, and developer tooling

ESLint, Oxlint, Prettier, formatter, doctor, configuration, and tooling contracts.

| ID    | Workflow                           | File                                                                                         |
| ----- | ---------------------------------- | -------------------------------------------------------------------------------------------- |
| 09.01 | Doctor barrel prop accuracy        | [`doctor-barrel-props.yml`](./doctor-barrel-props.yml)                                       |
| 09.02 | Doctor and lint SSR context        | [`doctor-lint-ssr-context.yml`](./doctor-lint-ssr-context.yml)                               |
| 09.03 | Doctor prop accuracy               | [`doctor-prop-accuracy.yml`](./doctor-prop-accuracy.yml)                                     |
| 09.04 | ESLint plugin consumer             | [`eslint-plugin-consumer.yml`](./eslint-plugin-consumer.yml)                                 |
| 09.05 | Lint reference coverage contracts  | [`lint-reference-coverage-contracts.yml`](./lint-reference-coverage-contracts.yml)           |
| 09.06 | Lint unused disable configuration  | [`lint-unused-disable-config.yml`](./lint-unused-disable-config.yml)                         |
| 09.07 | n8n adoption fixture               | [`n8n-adoption.yml`](./n8n-adoption.yml)                                                     |
| 09.08 | n8n script rule CLI controls       | [`n8n-script-rule-controls.yml`](./n8n-script-rule-controls.yml)                             |
| 09.09 | n8n whole corpus observer          | [`n8n-whole-corpus-observer.yml`](./n8n-whole-corpus-observer.yml)                           |
| 09.10 | Attribute ignoreTags reference     | [`patina-attribute-ignore-tags-reference.yml`](./patina-attribute-ignore-tags-reference.yml) |
| 09.11 | Patina native boolean protocols    | [`patina-native-boolean-protocol.yml`](./patina-native-boolean-protocol.yml)                 |
| 09.12 | Prettier plugin consumer           | [`prettier-plugin.yml`](./prettier-plugin.yml)                                               |
| 09.13 | Windows dynamic config consumer    | [`windows-config-consumer.yml`](./windows-config-consumer.yml)                               |
| 09.14 | Workspace route identity admission | [`workspace-route-identity.yml`](./workspace-route-identity.yml)                             |

## 10 Applications and UI packages

Musea, Fresco, playground, app E2E, UI packages, and browser-facing integrations.

| ID    | Workflow                            | File                                                                                   |
| ----- | ----------------------------------- | -------------------------------------------------------------------------------------- |
| 10.01 | Fresco                              | [`fresco.yml`](./fresco.yml)                                                           |
| 10.02 | Mounted runtime devtools comparison | [`mounted-runtime-devtools-comparison.yml`](./mounted-runtime-devtools-comparison.yml) |
| 10.03 | Musea accessibility and capture     | [`musea-browser.yml`](./musea-browser.yml)                                             |
| 10.04 | Musea glyph feature boundary        | [`musea-glyph-feature-boundary.yml`](./musea-glyph-feature-boundary.yml)               |

## 11 Security, maintenance, and documentation

Security audits, dependency upkeep, scheduled maintenance, docs, and repository hygiene.

| ID    | Workflow                      | File                                                                       |
| ----- | ----------------------------- | -------------------------------------------------------------------------- |
| 11.01 | Docs build                    | [`build-docs.yml`](./build-docs.yml)                                       |
| 11.02 | Clean obsolete merge groups   | [`cleanup-obsolete-merge-groups.yml`](./cleanup-obsolete-merge-groups.yml) |
| 11.03 | Deploy docs                   | [`deploy-docs.yml`](./deploy-docs.yml)                                     |
| 11.04 | Fuzz                          | [`fuzz.yml`](./fuzz.yml)                                                   |
| 11.05 | Language project fixture laws | [`language-project-fixtures.yml`](./language-project-fixtures.yml)         |
| 11.06 | Miri                          | [`miri.yml`](./miri.yml)                                                   |
| 11.07 | Production reach accounting   | [`production-reach-accounting.yml`](./production-reach-accounting.yml)     |
| 11.08 | qs runtime security contracts | [`qs-runtime-security.yml`](./qs-runtime-security.yml)                     |
| 11.09 | Sticky disk maintenance       | [`sticky-disk-maintenance.yml`](./sticky-disk-maintenance.yml)             |
| 11.10 | Generated VTS source census   | [`vts-source-census.yml`](./vts-source-census.yml)                         |
| 11.11 | Zizmor                        | [`zizmor.yml`](./zizmor.yml)                                               |
