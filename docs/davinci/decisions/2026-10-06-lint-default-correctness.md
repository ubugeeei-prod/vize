# Default lint correctness coverage

Issue [#8090](https://github.com/ubugeeei-prod/vize/issues/8090), paired [decision](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009162808).

The maintainer explicitly requested more correctness coverage without a lint configuration. Source base is actual main `143c1d4a9fbe6530cdd06f1f001b816f39c1d00a`. The CLI falls back to HappyPath; default/recommended/general-recommended are aliases. Configless LSP enables ecosystem support and selects Ecosystem. Strict/all select Opinionated; Nuxt inherits broad checks but accounts for auto-imports. The six named presets and selectors are retained.

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

The existing Ecosystem script rules already use the shared OXC parse. Additional enabled checks still incur visits; no speed improvement is claimed. No extra parser/pipeline stage, native type query, package dependency or instruction budget increase is introduced. Strict reactivity remains separately configured.

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

The strict source-CLI observer prepares62 observations: fourteen Vue carriers across configless/recommended/Ecosystem/Incremental plus six explicit-off controls. It persists raw process status/signal/error/stdout/stderr before reading files or asserting, then complete input bytes/hash, validates exact whole JSON and empty stderr, exit status and unchanged config/input. Build receipt binds the actual source revision, binary path/SHA and version. Existing tooling bootstrap and full differential artifact upload execute it without a new CI job/stage.

## Required acceptance and unfinished work

Prepared source, rustfmt and pure-reference inspection do not establish compiled Rust, CLI, editor or publication success. Require current-head Actions with authored Rust/API and LSP contracts plus source CLI62, protected full/instruction/performance suites, actual signed merge, existing Real Project Matrix Vue/Nuxt/Vite lint evidence, and next release/externally installed CLI verification. Keep #8090 open until that boundary. Measurements come from actual Actions/queue/Matrix rather than a local native build or process-startup timing claim. No waived audit, raised cap, missing-fixture skip or expected-output recapture is allowed.

Open P0 false-positive policies (export-in-script-setup #7934, prefer-computed #7899/#7901, native opt-ins and CSS conventions) remain outside this expansion. SSR #7982 is owned independently and already registered by HappyPath. Other potential default additions require their own concrete semantic/performance evidence; this is a first slice, not a claim that every available rule belongs in defaults.

## Source-derived help contract correction

Paired [qualification](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6009234947). Initial immutable source `05f6df9cd8e14669f871367a53345818f6f0b0d1` is retained. Before any native observation, direct producer inspection found that the initial macro-import whole vector incorrectly assumed HelpLevel::None clears script help. The unchanged rule attaches the literal help and script_rules::merge_script_result/severity::append_with_rule_overrides retain it; the CLI has no later normalization. Preserve `Remove the macro from the import statement. Compiler macros are auto-imported.` in the one macro-import API/CLI expected field. The whole JSON helper emits that field and still requires null help in every template control. This is an authored source-contract correction, with no captured runtime output or new help-level product behavior. All production bytes, full15 sources, ranges/counters/severities,62 process cases and whole serialized LSP vectors remain exact. Fresh successor runtime/CI acceptance is required; old results cannot be transferred.
