# Workflow catalog

GitHub only discovers workflow files directly under this directory, so the files
cannot be organized into subdirectories. This generated catalog provides the missing
navigation layer; reusable implementation belongs in `.github/actions/` or typed
drivers under `tools/`.

Regenerate with `node tools/commands/ci/workflow-catalog.ts`.

## Required and pull request gates

| Workflow                           | File                                                                       |
| ---------------------------------- | -------------------------------------------------------------------------- |
| Check Benchmark Gate               | [`check-bench.yml`](./check-bench.yml)                                     |
| Check                              | [`check.yml`](./check.yml)                                                 |
| Checker constructor emitter parity | [`checker-constructor-emitter.yml`](./checker-constructor-emitter.yml)     |
| App E2E                            | [`e2e.yml`](./e2e.yml)                                                     |
| Full qualification                 | [`full-qualification.yml`](./full-qualification.yml)                       |
| Full source CLI producer           | [`full-source-cli.yml`](./full-source-cli.yml)                             |
| Full tooling plan                  | [`full-tooling-plan.yml`](./full-tooling-plan.yml)                         |
| Full worker cancellation contracts | [`full-worker-contracts.yml`](./full-worker-contracts.yml)                 |
| Expression slow unit 10401 profile | [`fuzz-expression-10401-profile.yml`](./fuzz-expression-10401-profile.yml) |
| Fuzz                               | [`fuzz.yml`](./fuzz.yml)                                                   |
| Miri                               | [`miri.yml`](./miri.yml)                                                   |
| PR JS package checks               | [`pr-js-checks.yml`](./pr-js-checks.yml)                                   |
| PR performance observation         | [`pr-performance-observation.yml`](./pr-performance-observation.yml)       |
| Rust source checks                 | [`pr-rust-checks.yml`](./pr-rust-checks.yml)                               |
| Required Rust differential checks  | [`pr-rust-differential.yml`](./pr-rust-differential.yml)                   |
| Rust product checks                | [`pr-rust-products.yml`](./pr-rust-products.yml)                           |
| PR source checks                   | [`pr-source-checks.yml`](./pr-source-checks.yml)                           |
| PR source CLI producer             | [`pr-source-cli.yml`](./pr-source-cli.yml)                                 |
| Queue release gates                | [`queue-release-gates.yml`](./queue-release-gates.yml)                     |
| Seed PR Rust target                | [`rust-pr-target-seed.yml`](./rust-pr-target-seed.yml)                     |
| Title Policy                       | [`title-policy.yml`](./title-policy.yml)                                   |
| Zizmor                             | [`zizmor.yml`](./zizmor.yml)                                               |

## Release and public acceptance

| Workflow                                  | File                                                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------------------------ |
| Corsa package refresh                     | [`corsa-package-refresh.yml`](./corsa-package-refresh.yml)                                 |
| JSR package checks                        | [`jsr-package.yml`](./jsr-package.yml)                                                     |
| Public JSR types and runtime capabilities | [`jsr-public-types.yml`](./jsr-public-types.yml)                                           |
| pkg.pr.new                                | [`pkg-pr-new.yml`](./pkg-pr-new.yml)                                                       |
| Published plain-script import acceptance  | [`plain-script-import-public-acceptance.yml`](./plain-script-import-public-acceptance.yml) |
| Published JSX child values                | [`public-jsx-child-values.yml`](./public-jsx-child-values.yml)                             |
| Public native event casing                | [`public-native-events.yml`](./public-native-events.yml)                                   |
| Public typechecker consumer               | [`public-typechecker-consumer.yml`](./public-typechecker-consumer.yml)                     |
| Published original ambient template ref   | [`published-ambient-template-ref.yml`](./published-ambient-template-ref.yml)               |
| Published component ARIA                  | [`published-component-aria.yml`](./published-component-aria.yml)                           |
| Published computed event names            | [`published-computed-event-names.yml`](./published-computed-event-names.yml)               |
| Published original constructor 27         | [`published-constructor-original27.yml`](./published-constructor-original27.yml)           |
| Published human reports 11252 and 11253   | [`published-human-reports-11252-11253.yml`](./published-human-reports-11252-11253.yml)     |
| Published slot instance NoAny             | [`published-slot-instance-noany.yml`](./published-slot-instance-noany.yml)                 |
| Published slot provider controls          | [`published-slot-provider.yml`](./published-slot-provider.yml)                             |
| Release build cache                       | [`release-build-cache.yml`](./release-build-cache.yml)                                     |
| Release cadence                           | [`release-cadence.yml`](./release-cadence.yml)                                             |
| Release contracts                         | [`release-contracts.yml`](./release-contracts.yml)                                         |
| Release JSR                               | [`release-jsr.yml`](./release-jsr.yml)                                                     |
| Release smoke contracts                   | [`release-smoke-contracts.yml`](./release-smoke-contracts.yml)                             |
| Release verify                            | [`release-verify.yml`](./release-verify.yml)                                               |
| Release Zed host                          | [`release-zed-host.yml`](./release-zed-host.yml)                                           |
| Release                                   | [`release.yml`](./release.yml)                                                             |

## Performance and profiling

| Workflow                                            | File                                                                                     |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Benchmark                                           | [`benchmark.yml`](./benchmark.yml)                                                       |
| CI walltime comparison                              | [`ci-walltime-benchmark.yml`](./ci-walltime-benchmark.yml)                               |
| Cold typechecker scale                              | [`cold-typechecker-scale.yml`](./cold-typechecker-scale.yml)                             |
| Compile Benchmark                                   | [`compile-bench.yml`](./compile-bench.yml)                                               |
| Component binding view allocation profile           | [`component-binding-view-profile.yml`](./component-binding-view-profile.yml)             |
| Content Mapper Cold Start                           | [`content-mapper-cold-start.yml`](./content-mapper-cold-start.yml)                       |
| Content mapper span allocation                      | [`content-mapper-span-allocation.yml`](./content-mapper-span-allocation.yml)             |
| Criterion Bench                                     | [`criterion-bench.yml`](./criterion-bench.yml)                                           |
| Davinci Canon distinct-host scaling                 | [`davinci-canon-scaling.yml`](./davinci-canon-scaling.yml)                               |
| Davinci resource budgets                            | [`davinci-resource-budgets.yml`](./davinci-resource-budgets.yml)                         |
| Doctor filter benchmark                             | [`doctor-filter-bench.yml`](./doctor-filter-bench.yml)                                   |
| Document snapshot source profile                    | [`document-snapshot-source-profile.yml`](./document-snapshot-source-profile.yml)         |
| Public formatter profile acceptance                 | [`formatter-profiles-public.yml`](./formatter-profiles-public.yml)                       |
| Fresco dimension benchmark                          | [`fresco-dimension-bench.yml`](./fresco-dimension-bench.yml)                             |
| Level instruction counts                            | [`level-instruction-counts.yml`](./level-instruction-counts.yml)                         |
| Complete Level Memory Comparison                    | [`level-memory-bench.yml`](./level-memory-bench.yml)                                     |
| Same-wire LSP benchmark                             | [`lsp-dual-server-benchmark.yml`](./lsp-dual-server-benchmark.yml)                       |
| LSP marker scan allocation and residency comparison | [`lsp-marker-scan-scale.yml`](./lsp-marker-scan-scale.yml)                               |
| LSP source operations and allocations               | [`lsp-source-profile.yml`](./lsp-source-profile.yml)                                     |
| Profile allocation thread law                       | [`profile-allocation-thread-law.yml`](./profile-allocation-thread-law.yml)               |
| Script recovery clone cost                          | [`script-recovery-clone-cost.yml`](./script-recovery-clone-cost.yml)                     |
| Setup binding anchor allocation profile             | [`setup-binding-anchor-profile.yml`](./setup-binding-anchor-profile.yml)                 |
| Native SFC member phase comparison                  | [`sfc-member-profile.yml`](./sfc-member-profile.yml)                                     |
| Source operation profile                            | [`source-operation-profile.yml`](./source-operation-profile.yml)                         |
| String Storage Bench                                | [`string-storage-bench.yml`](./string-storage-bench.yml)                                 |
| Text boundary benchmark                             | [`text-boundary-bench.yml`](./text-boundary-bench.yml)                                   |
| Tool Benchmark                                      | [`tool-benchmark.yml`](./tool-benchmark.yml)                                             |
| Type admission scale axes                           | [`type-admission-scale.yml`](./type-admission-scale.yml)                                 |
| Type source snapshot benchmark                      | [`type-snapshot-bench.yml`](./type-snapshot-bench.yml)                                   |
| Type-check scaling                                  | [`typecheck-scaling.yml`](./typecheck-scaling.yml)                                       |
| Vue Benchmarks Current Qualification                | [`vue-benchmarks-current-qualification.yml`](./vue-benchmarks-current-qualification.yml) |
| Vue Benchmarks Replay                               | [`vue-benchmarks-replay.yml`](./vue-benchmarks-replay.yml)                               |

## Editors and language services

| Workflow                                  | File                                                                         |
| ----------------------------------------- | ---------------------------------------------------------------------------- |
| Packaged CSS hover visual evidence        | [`css-hover-visual.yml`](./css-hover-visual.yml)                             |
| Editor conformance                        | [`editor-conformance.yml`](./editor-conformance.yml)                         |
| Public editor installation custody        | [`editor-vite-stub-public.yml`](./editor-vite-stub-public.yml)               |
| LSP config startup diagnostics            | [`lsp-config-diagnostics.yml`](./lsp-config-diagnostics.yml)                 |
| Current-source LSP baseline               | [`lsp-current-baseline.yml`](./lsp-current-baseline.yml)                     |
| Focused real-project LSP qualification    | [`lsp-focused-projects.yml`](./lsp-focused-projects.yml)                     |
| JSX intrinsic hover observations          | [`lsp-jsx-intrinsic-hover.yml`](./lsp-jsx-intrinsic-hover.yml)               |
| LSP native diagnostic coverage            | [`lsp-native-diagnostic-coverage.yml`](./lsp-native-diagnostic-coverage.yml) |
| Native diagnostic warm-up                 | [`lsp-native-diagnostic-warmup.yml`](./lsp-native-diagnostic-warmup.yml)     |
| Native LSP overlay custody                | [`lsp-overlay-custody.yml`](./lsp-overlay-custody.yml)                       |
| LSP independent overlay graphs            | [`lsp-overlay-graph-reuse.yml`](./lsp-overlay-graph-reuse.yml)               |
| Shipped Linux LSP resource comparison     | [`lsp-release-resources.yml`](./lsp-release-resources.yml)                   |
| LSP indexed scheduler source cost carrier | [`lsp-scheduler-cost-carrier.yml`](./lsp-scheduler-cost-carrier.yml)         |
| Tag prefix classification observations    | [`lsp-tag-prefix-classification.yml`](./lsp-tag-prefix-classification.yml)   |
| LSP vs Vue language tools                 | [`lsp-vs-volar.yml`](./lsp-vs-volar.yml)                                     |
| Native named SFC value hover              | [`named-sfc-value-hover.yml`](./named-sfc-value-hover.yml)                   |
| Router installed editor artifact          | [`router-editor-artifact.yml`](./router-editor-artifact.yml)                 |
| TypeScript plugin packed consumer         | [`typescript-plugin.yml`](./typescript-plugin.yml)                           |
| Published TypeScript provider acceptance  | [`typescript-provider-public.yml`](./typescript-provider-public.yml)         |
| Platform VSIX native projection           | [`typescript-provider-vsix.yml`](./typescript-provider-vsix.yml)             |
| Volar Plugin                              | [`volar-plugin.yml`](./volar-plugin.yml)                                     |
| VS Code VSIX validation                   | [`vscode-vsix-validation.yml`](./vscode-vsix-validation.yml)                 |
| VS Code workspace server                  | [`vscode-workspace-server.yml`](./vscode-workspace-server.yml)               |

## Compiler and native correctness

| Workflow                                   | File                                                                                     |
| ------------------------------------------ | ---------------------------------------------------------------------------------------- |
| Async component render ancestry            | [`async-component-ancestry.yml`](./async-component-ancestry.yml)                         |
| Binding origin source laws                 | [`binding-origin-laws.yml`](./binding-origin-laws.yml)                                   |
| Canon component casing accuracy            | [`canon-component-casing.yml`](./canon-component-casing.yml)                             |
| Canon helper native baseline               | [`canon-helper-native-baseline.yml`](./canon-helper-native-baseline.yml)                 |
| Canon instance directive ownership         | [`canon-instance-directives.yml`](./canon-instance-directives.yml)                       |
| Canon slot constructor admission           | [`canon-slot-constructor.yml`](./canon-slot-constructor.yml)                             |
| Canon slot provider ownership              | [`canon-slot-provider-any.yml`](./canon-slot-provider-any.yml)                           |
| Component input diagnostic ownership       | [`component-input-diagnostic-ownership.yml`](./component-input-diagnostic-ownership.yml) |
| Component input projection packets         | [`component-input-projection-packets.yml`](./component-input-projection-packets.yml)     |
| Component scope binding allocation profile | [`component-scope-binding.yml`](./component-scope-binding.yml)                           |
| Cross-file finalization work comparison    | [`cross-file-finalization-compare.yml`](./cross-file-finalization-compare.yml)           |
| Cross-file finalization order              | [`cross-file-finalization-order.yml`](./cross-file-finalization-order.yml)               |
| Cross-file dependency profiling            | [`cross-file-finalization.yml`](./cross-file-finalization.yml)                           |
| Cross-file script coordinate packets       | [`cross-file-script-offset.yml`](./cross-file-script-offset.yml)                         |
| Cross-file SFC effect graphs               | [`cross-file-sfc-effects.yml`](./cross-file-sfc-effects.yml)                             |
| Cross-file retained import updates         | [`cross-file-source-updates.yml`](./cross-file-source-updates.yml)                       |
| Cross-file WASM effect grammars            | [`cross-file-wasm-effects.yml`](./cross-file-wasm-effects.yml)                           |
| Canonical Davinci corpus                   | [`davinci-canonical-corpus.yml`](./davinci-canonical-corpus.yml)                         |
| Davinci Incremental                        | [`davinci-incremental.yml`](./davinci-incremental.yml)                                   |
| Davinci Kani                               | [`davinci-kani.yml`](./davinci-kani.yml)                                                 |
| Davinci Plugin SDK                         | [`davinci-plugin-sdk.yml`](./davinci-plugin-sdk.yml)                                     |
| Davinci reference backends                 | [`davinci-reference.yml`](./davinci-reference.yml)                                       |
| Davinci unused bindings                    | [`davinci-unused-bindings.yml`](./davinci-unused-bindings.yml)                           |
| Optional inject fallback contracts         | [`defaulted-inject.yml`](./defaulted-inject.yml)                                         |
| Decoded loop bindings                      | [`for-bindings.yml`](./for-bindings.yml)                                                 |
| JSX authored native modules                | [`jsx-authored-native-modules.yml`](./jsx-authored-native-modules.yml)                   |
| JSX backend source origins                 | [`jsx-backend-origins.yml`](./jsx-backend-origins.yml)                                   |
| Level deletion readiness                   | [`level-deletion-readiness.yml`](./level-deletion-readiness.yml)                         |
| Level dependencies                         | [`level-dependencies.yml`](./level-dependencies.yml)                                     |
| Level HTML content model                   | [`level-html-content-model.yml`](./level-html-content-model.yml)                         |
| Native beta diagnostic DTO                 | [`native-beta-diagnostic-dto.yml`](./native-beta-diagnostic-dto.yml)                     |
| Native release token layout                | [`native-build-token-layout.yml`](./native-build-token-layout.yml)                       |
| Native checker CPU attribution             | [`native-checker-cpu-metadata.yml`](./native-checker-cpu-metadata.yml)                   |
| Native image-alt query source evidence     | [`native-image-alt-query.yml`](./native-image-alt-query.yml)                             |
| Native operational incident trace          | [`native-ops-incident.yml`](./native-ops-incident.yml)                                   |
| SFC runtime option transport               | [`native-sfc-runtime-options.yml`](./native-sfc-runtime-options.yml)                     |
| SFC runtime version selection              | [`native-sfc-runtime-version.yml`](./native-sfc-runtime-version.yml)                     |
| Native Smoke                               | [`native-smoke.yml`](./native-smoke.yml)                                                 |
| Recovery normal codegen                    | [`recovery-normal-codegen.yml`](./recovery-normal-codegen.yml)                           |
| Native SFC optional lifecycle diagnostic   | [`sfc-optional-lifecycle.yml`](./sfc-optional-lifecycle.yml)                             |
| Native parsed SFC attributes               | [`sfc-parse-attributes.yml`](./sfc-parse-attributes.yml)                                 |
| Historical SFC Parse Replay                | [`sfc-parse-replay.yml`](./sfc-parse-replay.yml)                                         |
| Public mapper comparison                   | [`template-alias-paired-11822.yml`](./template-alias-paired-11822.yml)                   |
| Template compiler global policy            | [`template-global-policy.yml`](./template-global-policy.yml)                             |
| Vapor literal object styles                | [`vapor-literal-object-styles.yml`](./vapor-literal-object-styles.yml)                   |

## Packages and ecosystem

| Workflow                                 | File                                                                                         |
| ---------------------------------------- | -------------------------------------------------------------------------------------------- |
| Babel plugin host contracts              | [`babel-plugin-hosts.yml`](./babel-plugin-hosts.yml)                                         |
| Babel rc9 ABI frontier                   | [`babel-rc9-frontier.yml`](./babel-rc9-frontier.yml)                                         |
| Content Mapper Conformance               | [`content-mapper-conformance.yml`](./content-mapper-conformance.yml)                         |
| Consumer instance globals capture        | [`content-mapper-instance-globals.yml`](./content-mapper-instance-globals.yml)               |
| Content Mapper Source Case Jobs          | [`content-mapper-reported-q16.yml`](./content-mapper-reported-q16.yml)                       |
| Content Mapper worker stack              | [`content-mapper-worker-stack.yml`](./content-mapper-worker-stack.yml)                       |
| Doctor barrel prop accuracy              | [`doctor-barrel-props.yml`](./doctor-barrel-props.yml)                                       |
| Doctor and lint SSR context              | [`doctor-lint-ssr-context.yml`](./doctor-lint-ssr-context.yml)                               |
| Doctor prop accuracy                     | [`doctor-prop-accuracy.yml`](./doctor-prop-accuracy.yml)                                     |
| ESLint plugin consumer                   | [`eslint-plugin-consumer.yml`](./eslint-plugin-consumer.yml)                                 |
| Fresco                                   | [`fresco.yml`](./fresco.yml)                                                                 |
| Lint reference coverage contracts        | [`lint-reference-coverage-contracts.yml`](./lint-reference-coverage-contracts.yml)           |
| Lint unused disable configuration        | [`lint-unused-disable-config.yml`](./lint-unused-disable-config.yml)                         |
| Musea accessibility and capture          | [`musea-browser.yml`](./musea-browser.yml)                                                   |
| Musea glyph feature boundary             | [`musea-glyph-feature-boundary.yml`](./musea-glyph-feature-boundary.yml)                     |
| n8n adoption fixture                     | [`n8n-adoption.yml`](./n8n-adoption.yml)                                                     |
| n8n script rule CLI controls             | [`n8n-script-rule-controls.yml`](./n8n-script-rule-controls.yml)                             |
| n8n whole corpus observer                | [`n8n-whole-corpus-observer.yml`](./n8n-whole-corpus-observer.yml)                           |
| Nuxt generated declaration membership    | [`nuxt-generated-membership.yml`](./nuxt-generated-membership.yml)                           |
| Nuxt public runtime dependencies         | [`nuxt-public-dependencies.yml`](./nuxt-public-dependencies.yml)                             |
| Nuxt scoped style build                  | [`nuxt-style-build.yml`](./nuxt-style-build.yml)                                             |
| Nuxt 2 Bridge host runtimes              | [`nuxt2-bridge-module-build.yml`](./nuxt2-bridge-module-build.yml)                           |
| Nuxt 2 module build                      | [`nuxt2-module-build.yml`](./nuxt2-module-build.yml)                                         |
| Nuxt 3 module build                      | [`nuxt3-module-build.yml`](./nuxt3-module-build.yml)                                         |
| Parcel transformer                       | [`parcel-transformer.yml`](./parcel-transformer.yml)                                         |
| Public attribute ignore acceptance       | [`patina-attribute-ignore-public.yml`](./patina-attribute-ignore-public.yml)                 |
| Attribute ignoreTags reference           | [`patina-attribute-ignore-tags-reference.yml`](./patina-attribute-ignore-tags-reference.yml) |
| Public strict boolean runtime acceptance | [`patina-boolean-public.yml`](./patina-boolean-public.yml)                                   |
| Patina native boolean protocols          | [`patina-native-boolean-protocol.yml`](./patina-native-boolean-protocol.yml)                 |
| Prettier plugin consumer                 | [`prettier-plugin.yml`](./prettier-plugin.yml)                                               |
| Rspack SFC HMR                           | [`rspack-hmr.yml`](./rspack-hmr.yml)                                                         |
| Rspack external block ranges             | [`rspack-src-block-ranges.yml`](./rspack-src-block-ranges.yml)                               |
| Turbopack consumers                      | [`turbopack-consumers.yml`](./turbopack-consumers.yml)                                       |
| Vite+ Type Compatibility                 | [`vite-plus-type-compat.yml`](./vite-plus-type-compat.yml)                                   |
| Vite precompile cache identities         | [`vite-precompile-cache.yml`](./vite-precompile-cache.yml)                                   |
| Vite relative style origins              | [`vite-style-relative-imports.yml`](./vite-style-relative-imports.yml)                       |
| Vite+ configuration consumers            | [`viteplus-config-consumers.yml`](./viteplus-config-consumers.yml)                           |
| Vite+ lint configuration consumers       | [`viteplus-lint-consumers.yml`](./viteplus-lint-consumers.yml)                               |
| Vite+ task cache consumers               | [`viteplus-task-cache.yml`](./viteplus-task-cache.yml)                                       |
| Vitest persistent Vue modules            | [`vitest-module-cache.yml`](./vitest-module-cache.yml)                                       |
| Vitest project configuration consumers   | [`vitest-project-extends.yml`](./vitest-project-extends.yml)                                 |

## Maintenance and documentation

| Workflow                                 | File                                                                                         |
| ---------------------------------------- | -------------------------------------------------------------------------------------------- |
| Ambient template ref ownership           | [`ambient-template-ref-ownership.yml`](./ambient-template-ref-ownership.yml)                 |
| SFC asset import bindings                | [`asset-import-bindings.yml`](./asset-import-bindings.yml)                                   |
| Docs build                               | [`build-docs.yml`](./build-docs.yml)                                                         |
| Canonical corpus objects                 | [`canonical-corpus-objects.yml`](./canonical-corpus-objects.yml)                             |
| Clean obsolete merge groups              | [`cleanup-obsolete-merge-groups.yml`](./cleanup-obsolete-merge-groups.yml)                   |
| Compact Collections                      | [`compact-collections.yml`](./compact-collections.yml)                                       |
| Compiler options snapshot                | [`compiler-options-snapshot.yml`](./compiler-options-snapshot.yml)                           |
| CSS color declaration admission          | [`css-color-declaration-admission.yml`](./css-color-declaration-admission.yml)               |
| CSS color literal hover                  | [`css-color-literal-hover.yml`](./css-color-literal-hover.yml)                               |
| Native CSS source maps                   | [`css-compile-source-maps.yml`](./css-compile-source-maps.yml)                               |
| CSS engine forks                         | [`css-engine-vendor-candidate.yml`](./css-engine-vendor-candidate.yml)                       |
| Deploy docs                              | [`deploy-docs.yml`](./deploy-docs.yml)                                                       |
| Empty generic public contract            | [`empty-generic-public-contract.yml`](./empty-generic-public-contract.yml)                   |
| Expression overlapping speculation audit | [`expression-overlap-audit.yml`](./expression-overlap-audit.yml)                             |
| JS install dependency closure            | [`js-install-closure.yml`](./js-install-closure.yml)                                         |
| L1 document special text                 | [`l1-document-special-text.yml`](./l1-document-special-text.yml)                             |
| Language project fixture laws            | [`language-project-fixtures.yml`](./language-project-fixtures.yml)                           |
| Mounted runtime devtools comparison      | [`mounted-runtime-devtools-comparison.yml`](./mounted-runtime-devtools-comparison.yml)       |
| Narrowed slot host guard                 | [`narrowed-slot-host-guard.yml`](./narrowed-slot-host-guard.yml)                             |
| Partition weight work                    | [`partition-weight-work.yml`](./partition-weight-work.yml)                                   |
| Persistent batch storage                 | [`persistent-batch-storage.yml`](./persistent-batch-storage.yml)                             |
| Plain-script import ref runtime          | [`plain-script-import-ref.yml`](./plain-script-import-ref.yml)                               |
| Leading newline runtime                  | [`pre-textarea-first-newline.yml`](./pre-textarea-first-newline.yml)                         |
| Production reach accounting              | [`production-reach-accounting.yml`](./production-reach-accounting.yml)                       |
| Provide type assertion whole packets     | [`provide-type-assertion.yml`](./provide-type-assertion.yml)                                 |
| qs runtime security contracts            | [`qs-runtime-security.yml`](./qs-runtime-security.yml)                                       |
| Real Project Matrix                      | [`real-project-matrix.yml`](./real-project-matrix.yml)                                       |
| Published router navigation matrix       | [`router-experimental-navigation.yml`](./router-experimental-navigation.yml)                 |
| Router preview source provider           | [`router-preview-conformance.yml`](./router-preview-conformance.yml)                         |
| Scoped nesting specificity               | [`scoped-nesting-specificity.yml`](./scoped-nesting-specificity.yml)                         |
| Script recovery cost                     | [`script-recovery-cost.yml`](./script-recovery-cost.yml)                                     |
| Retained source-line replay              | [`source-symbolizer-retained.yml`](./source-symbolizer-retained.yml)                         |
| Source symbolizer smoke                  | [`source-symbolizer-smoke.yml`](./source-symbolizer-smoke.yml)                               |
| Sticky disk maintenance                  | [`sticky-disk-maintenance.yml`](./sticky-disk-maintenance.yml)                               |
| Type-check corpus pair                   | [`typecheck-corpus-pair.yml`](./typecheck-corpus-pair.yml)                                   |
| Typecheck vs vue-tsc                     | [`typecheck-vs-vue-tsc.yml`](./typecheck-vs-vue-tsc.yml)                                     |
| WSL2 type-check qualification            | [`typecheck-wsl2.yml`](./typecheck-wsl2.yml)                                                 |
| Typechecker cache custody                | [`typechecker-cache-custody.yml`](./typechecker-cache-custody.yml)                           |
| Type checker native backend phases       | [`typechecker-native-phases.yml`](./typechecker-native-phases.yml)                           |
| Native syntax and semantic completion    | [`typechecker-syntax-semantic-completion.yml`](./typechecker-syntax-semantic-completion.yml) |
| Typed context undefined contracts        | [`typed-context-undefined.yml`](./typed-context-undefined.yml)                               |
| Unicode prop projection                  | [`unicode-prop-projection.yml`](./unicode-prop-projection.yml)                               |
| Void build                               | [`void-build.yml`](./void-build.yml)                                                         |
| Generated VTS source census              | [`vts-source-census.yml`](./vts-source-census.yml)                                           |
| Vue 2 editor diagnostic context          | [`vue2-editor-diagnostic-context.yml`](./vue2-editor-diagnostic-context.yml)                 |
| Native Vue 2 IR template target          | [`vue2-ir-template.yml`](./vue2-ir-template.yml)                                             |
| VueFes SSR style dependencies            | [`vuefes-ssr-style-dependencies.yml`](./vuefes-ssr-style-dependencies.yml)                   |
| Windows dynamic config consumer          | [`windows-config-consumer.yml`](./windows-config-consumer.yml)                               |
| Workspace route identity admission       | [`workspace-route-identity.yml`](./workspace-route-identity.yml)                             |
