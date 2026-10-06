# Default lint correctness coverage

Issue [#8090](https://github.com/ubugeeei-prod/vize/issues/8090), paired [decision](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009162808).

The maintainer explicitly requested more correctness coverage without a lint configuration. Initial source base is actual main `143c1d4a9fbe6530cdd06f1f001b816f39c1d00a`. The CLI falls back to HappyPath; default/recommended/general-recommended are aliases. Configless LSP enables ecosystem support and selects Ecosystem. Strict/all select Opinionated; Nuxt inherits broad checks but accounts for auto-imports. The six named presets and selectors are retained.

## Approved first policy slice

- `vue/no-deprecated-v-bind-sync`
- `vue/no-deprecated-v-on-native-modifier`
- `vue/no-deprecated-slot-scope-attribute`
- `vue/no-deprecated-scope-attribute`

Enable these existing template syntax checks in HappyPath and Essential and their inherited presets. Existing severity is Error, not a style warning. These removed constructs can silently break Vue 3 bindings/listeners/slots. Preserve the exact Vue 3 dialect guards, existing diagnostic messages, rule implementation and no-fix metadata; existing opt-in selection remains deduplicated. Explicit off still wins and Incremental remains empty.

The semantic authorities are the [Vue v-model migration](https://v3-migration.vuejs.org/breaking-changes/v-model.html), [native modifier removal](https://v3-migration.vuejs.org/breaking-changes/v-on-native-modifier-removed.html), upstream [slot-scope](https://eslint.vuejs.org/rules/no-deprecated-slot-scope-attribute.html) and [scope](https://eslint.vuejs.org/rules/no-deprecated-scope-attribute.html) Essential rules. The corpus consists of authored minimal full carriers and controls derived from these contracts and the already checked-in implementations, rather than copied third-party issue inputs. Original implementation authority at the source base:

- `crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs`: SHA256 `63973b6ff2907df72df326cc76b69a62cb14fee2cd5c7c3f299ed5575c4ba542`
- `crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs`: SHA256 `4d66d0659993afc3ac939a0f2aa873eeea5223f254b482552f5dc2073bc17414`
- `crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs`: SHA256 `4f027be43dade60c0e33723ad998d8280658d34ced061c7ea9cdaca353792039`
- `crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs`: SHA256 `858159bf4eccc3c9b3da7147dcd4e969a02245311c4dfa1b42ea92d2d6a70719`

## Ecosystem omission

The template registry already inherits HappyPath, but two static script membership arrays omitted Ecosystem. Repair the metadata so configuration-free editor lint retains these exact existing default script rules:

- `script/no-async-in-computed`
- `script/prefer-import-from-vue`
- `script/no-internal-imports`
- `script/no-import-compiler-macros`
- `script/no-reserved-identifiers`
- `script/no-reserved-keys`
- `script/no-dupe-keys`
- `script/no-side-effects-in-computed-properties`
- `script/no-arrow-functions-in-watch`
- `script/no-potential-component-option-typo`
- `script/return-in-computed-property`
- `script/require-prop-type-constructor`
- `script/no-use-computed-property-like-method`
- `script/no-required-prop-with-default`
- `script/valid-next-tick`
- `script/valid-define-options`
- `script/no-reserved-props`
- `script/return-in-emits-validator`
- `script/valid-define-props`
- `script/valid-define-emits`
- `script/require-valid-default-prop`
- `script/no-ref-as-operand`
- `script/no-duplicate-attr-inheritance`
- `script/no-multiple-slot-args`
- `script/no-unstable-nested-components`

Paired [cost qualification](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009294623): matching Ecosystem script rules already shared the OXC parse, but the former ecosystem-only byte prefilter could skip AST construction for ordinary scripts. Restoring the 25 default checks removes that early return; the existing no-duplicate-attr-inheritance prefilter is true, so ordinary scripts can newly execute the existing single shared OXC pass. This is newly enabled work with a real cost, which requires actual protected and real-project/editor measurement. No zero-cost or speed improvement is claimed. No extra parser/pipeline stage, duplicate parse, native type query, package dependency or instruction budget increase is introduced. Strict reactivity remains separately configured.

## Authored conservation and observations

| Preset              | Incoming total | Authored total |
| ------------------- | -------------: | -------------: |
| HappyPath           |            114 |            118 |
| Essential           |             56 |             60 |
| Ecosystem           |             97 |            126 |
| Opinionated         |            171 |            175 |
| Nuxt                |            173 |            177 |
| Incremental         |              0 |              0 |
| Opt-in construction |            104 |            104 |

The checked-in membership snapshot is authored by adding only these four template entries and restoring the exact25 script entries in original registration order. Removing the approved additions restores every prior ordered list. The eslint rule map changes only the same memberships, retaining severity/status/targets and all other entries. No runtime snapshot was recaptured. The whole incoming 350-line canonical record is conserved with one appended owned clause; budgets remain intact. The generated Patina consumer census adds only the dedicated API consumer row.

The 15 complete source carriers live in `tests/_fixtures/lint-default-correctness/`, each duplicated byte-exact in its fixed corpus record. Whole API results cover every diagnostic field and every counter. Whole LSP serialized Diagnostic vectors cover implicit Ecosystem and explicit recommended, including help/documentation links/ranges and UTF-16 coordinates. Valid v-model/event/slot, native HTML scope, comment text, compiler macro import/comment, UTF-8/CRLF, explicit-off, empty Incremental and petite-vue controls are retained. No classifier/filter hides extra findings.

The strict source-CLI observer preserves the original62 observations: fourteen Vue carriers across configless/recommended/Ecosystem/Incremental plus six explicit-off controls. It persists raw process status/signal/error/stdout/stderr before reading files or asserting, then complete input bytes/hash, validates exact whole JSON and empty stderr, exit status and unchanged config/input. Build receipt binds the actual source revision, binary path/SHA and version. Existing tooling bootstrap and full differential artifact upload execute it without a new CI job/stage.

## Required acceptance and unfinished work

Prepared source, rustfmt and pure-reference inspection do not establish compiled Rust, CLI, editor or publication success. Require current-head Actions with authored Rust/API and LSP contracts plus current source CLI80, protected full/instruction/performance suites, actual signed merge, existing Real Project Matrix Vue/Nuxt/Vite lint evidence, and next release/externally installed CLI verification. Keep #8090 open until that boundary. Measurements come from actual Actions/queue/Matrix rather than a local native build or process-startup timing claim. No waived audit, raised cap, missing-fixture skip or expected-output recapture is allowed.

Open P0 false-positive policies (export-in-script-setup #7934, prefer-computed #7899/#7901, native opt-ins and CSS conventions) remain outside this expansion. SSR #7982 is owned independently and already registered by HappyPath. Other potential default additions require their own concrete semantic/performance evidence; this is a first slice, not a claim that every available rule belongs in defaults.

## Source-derived help contract correction

Paired [qualification](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009234947). Initial immutable source `05f6df9cd8e14669f871367a53345818f6f0b0d1` is retained. Before any native observation, direct producer inspection found that the initial macro-import whole vector incorrectly assumed HelpLevel::None clears script help. The unchanged rule attaches the literal help and script_rules::merge_script_result/severity::append_with_rule_overrides retain it; the CLI has no later normalization. Preserve `Remove the macro from the import statement. Compiler macros are auto-imported.` in the one macro-import API/CLI expected field. The whole JSON helper emits that field and still requires null help in every template control. This is an authored source-contract correction, with no captured runtime output or new help-level product behavior. All production bytes, full15 sources, ranges/counters/severities,62 process cases and whole serialized LSP vectors remain exact. Fresh successor runtime/CI acceptance is required; old results cannot be transferred.

## Complete existing warning baseline

Paired [source-run correction](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009438999). Actual source run [37413081112](https://github.com/ubugeeei-prod/vize/actions/runs/37413081112) rejected the authored complete API/LSP/CLI vectors for the unchanged full-modifiers carrier: the initial oracle omitted the pre-existing vue/v-on-style long-form warning. Its producer remains byte-exact to actual main, SHA256 `0e5efb4ea4d06462604f7d0d5289ead67bca744ff1f7921fd670cae1ca30cc93`. Preserve the entire source-derived warning at bytes [50,88), its original message/backticks, raw help and complete Use shorthand syntax Fix replacing that assignment with `@activate.native.stop="onActivate"`. Direct ctx.report retains its help and Fix. The whole default vector is sync error, style warning, native error, with two errors and one warning; LSP severity2 and CLI JSON severity1 retain all original metadata. Four promoted rules explicitly off still leaves the one existing warning, while Incremental stays empty. CLI exit status derives from errors, not total findings.

All15 whole source carriers, approved four/25 producer bytes and62 process cases are conserved. Complete vectors are derived from unchanged source, with no runtime recapture, dropped finding or product-rule workaround. The historical ffeed source-only peer authenticates policy and custody but grants no complete-oracle/runtime acceptance. Fresh repaired source Actions remain mandatory. Only the affected29 preset cells in each of the five existing localized rule tables are synchronized to the authoritative ordered membership; every other cell, description, severity, link and incoming line is conserved. This also repairs stale localized membership for those same rules. Current CI/security/protected, actual merge, real project/cost evidence, release and installed CLI remain unfinished.

Source-only [unused-field qualification](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009524704): independent review found that macro-import baseline fields were empty although the four promoted template rules do not affect its existing script error. Retain the whole unchanged source-authored macro API/LSP vector in those fields. All15 baselines now retain existing findings; the executed first-six explicit-off controls and all62 process cases/expected runtime outputs are unchanged. No additional case, source input, product producer or native recapture is introduced; fresh exact successor acceptance remains required.

## Configured Vue version host repair

Paired [configured-host decision](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009761251). Independent review found that ordinary SFC TemplateRuleEnv remains VueDialect::Vue even under requested V2/V2_7. Extend only the existing legacy compatibility suppression with the four newly defaulted names. Vue3/configless results, standalone Petite routing, all other compatibility names and producers remain unchanged.

Maestro discarded the existing Carton loader-returned LinterFeatureFlags and never forwarded its Vue version into Patina. Keep those tiny resolved flags alongside the same folder snapshot and return them with the already-resolved lint settings; forward vue_version into the existing constructor. For outside-folder/global settings, store the flags beside LinterConfig under its original lock and derive them from the same loaded ConfigFeatureFlags with existing L0 precedence. This adds no parse, IO, lock, context scan, native query or pipeline stage. Seven existing test consumers only gain a tuple placeholder; every old assertion remains exact. lsp_priority confirmed these paths are disjoint from #8095.

Retain all15 carrier bytes and the exact original62 CLI scenarios. Append18 configured2/2.7/3 controls for80 total. Whole configured API and editor vectors cover all14 Vue carriers: legacy retains complete baseline, explicit3 retains complete default. Distinct legacy, modern and configless folder contexts plus outside-folder global fallback must retain complete vectors without sibling policy leakage. Existing Vue1/0 grammar completeness and editor Vapor forwarding remain unfinished; these are configured host/version controls, not a historical grammar claim.

Historical Check37415339973 at b6 passes four Rust shards/native/clippy/build and sourceCLI62. The complete130090-byte artifact SHA531bc90da9b6ba6852863ef888b709ed448d9bed40995c3abc98ae14c4e2e57e independently matches every raw stream/status/wholeJSON/input/config and binds hosted8c6daf (parents143+b6, exacttree128f431). Overall Check rejects old-main audit and gives no new-host/protected acceptance. Actual signed8080 main48cb merged05:04:12Z; the genuine replay onto signed48cb must preserve every incoming decision and same resolved flags. Fresh exact source Rust/API/LSP/CLI80/security, unchanged protected full/cost/real-project gates, actual merge and installed release remain mandatory. No waiver, budget increase, local native build or additional approval fence.

Paired [test setup repair](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009907302): source Check37417475543 at1e75 passes all4 tooling jobs/CLI80 but Clippy rejects the added std::format! folder label before Rust execution. Replace that call with the same three authored literal labels, without a lint allowance. The derived global-fallback test must load the primary project config before setting workspace folders, matching actual server/handlers.rs initialize. Keep every full expected vector, matching-folder law and all producer/corpus/80 process bytes unchanged; this repairs preparation rather than weakening the oracle. Failed source and positive CLI remain historical. Fresh exact repaired Rust/API/LSP/source/protected/actual merge/cost/project/installed acceptance stays mandatory.

## Actual merged-main integration

Paired [integration decision](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6010137268). Boolean fix #8078 actually merged at05:37:19Z as signed de40f, sole48cb parent, exact7349 source tree. Incorporate that actual main into this existing PR with genuine ancestry. Resolve the only canonical record conflict by relocating the owned default clause, conserving every whole incoming350 byte prefix and every incoming boolean source/control; regenerate the official combined Patina consumer census. All15 carriers, three policy producers, resolved workspace host flags, whole API/LSP vectors and80 CLI laws remain exactc5a.

Current c5a Check37418248601 passes all4 Rust shards and4 tooling jobs; retained XML independently authenticates all8 dedicated API/LSP laws and4 unchanged workspace settings tests, with16,313 unique reported source tests and no failure/error/duplicate. Whole currentCLI80 artifact163607B SHAbb5445ca81b1b69e4e61bfbf39737dfc2a36ef942c014509ab0644669b7028b2 binds hostedbcd8c (parents48cb+c5a, exact52a097 tree), binary417a277f, every raw stream/process/input/config and full JSON. Canonical production reach is still pending, so current completeCI acceptance is not claimed or transferred. The configured source peer seals all47 objects/full inverses atc5a; successor runtime/protected/cost/merge/publication acceptance remains fresh.

Real JavaScript #8102 independently exposes the same missing ESSENTIAL NoDupeKeys ecosystem producer on48cb with editor:true/lint:true/typecheck:false. Retain the actual failure and complete original oracle; compose the existing restored producer through a genuine dependency when required. No ecosystem:false workaround, empty-result acceptance or new classifier/parser patch. Ordinary-script parsing cost still needs actual protected and real-project/editor evidence. Keep #8090 open through the next finite release and installed80 verification; no new approval fence.
