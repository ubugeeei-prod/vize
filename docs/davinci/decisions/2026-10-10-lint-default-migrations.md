# Second default Vue 3 migration slice

Issue [#8090](https://github.com/ubugeeei-prod/vize/issues/8090), paired [decision](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6094237805). Source base is actual main `e5702be6d0119a716499827fa7b918ce8c9dcbae`.

The maintainer requested more enabled rules per preset. The [first slice](./2026-10-06-lint-default-correctness.md) already enables four removed Vue 3 constructs and retains Ecosystem script checks. This second, independently reviewable slice enables two existing Essential errors:

- `vue/no-deprecated-v-on-number-modifiers`
- `vue/no-deprecated-functional-template`

HappyPath and Essential register them; Opinionated, Ecosystem and Nuxt inherit them. Their existing messages, locations, Error severity and no-fix metadata are retained. Numeric modifiers no longer express the intended key filter in Vue 3; functional SFC templates were removed. Authority: the Vue [keyCode migration](https://v3-migration.vuejs.org/breaking-changes/keycode-modifiers.html), [functional component migration](https://v3-migration.vuejs.org/breaking-changes/functional-components.html), and the corresponding [numeric](https://eslint.vuejs.org/rules/no-deprecated-v-on-number-modifiers.html) and [functional](https://eslint.vuejs.org/rules/no-deprecated-functional-template.html) Essential rules.

## Compatibility and work ownership

Explicit Vue 2/2.7 compatibility suppresses both checks, including an explicitly selected rule. Named/alphanumeric key modifiers, ordinary templates, a native element's unrelated functional attribute, comment text and `v-pre` remain complete controls. Standalone petite-vue remains inert for the selected migration rules. Explicit off wins; Incremental remains empty. Opt-in registration remains available and idempotent.

The n8n adoption lane owns its valid-v-slot producers, full51/three-option/six-scope configuration and original fixtures. This change modifies none of them. The new corpus additionally executes the exact existing explicit51 projection on its own twelve authored carriers, asserting whole packets and no default leakage. That supplement does not replace the original full51 adoption qualification, unretired omissions, or the type-aware Vue ESLint boundary. No n8n upstream action is authorized or performed.

## Parse reuse and cost

The functional rule is listed in the existing shared SFC descriptor demand. It consumes the descriptor already prepared by the lint engine and exits before descriptor work when explicitly disabled. No extra pipeline stage, duplicate ordinary-route parser, serialization, dependency or type query is introduced. Numeric checking adds an existing modifier loop; functional checking adds an existing descriptor attribute lookup. Newly enabled work has a cost. No zero-cost or speed improvement is claimed; unchanged protected instruction/performance ceilings and actual editor/project gates must qualify it. No budget may be raised to admit the change.

## Authored complete conservation

`tests/_fixtures/lint-default-migrations/` contains twelve full SFC carriers, fixed in both source files and the authored JSON record. Complete API results include every finding, severity, range, help, fix, label and counter. The longhand event carrier retains its pre-existing style warning and full fix. Baselines reconstruct the incoming membership by disabling exactly the two promoted names; they are not captured previous-version runtime outputs. The configured Vue2/2.7 results retain those full baselines.

The preset membership snapshot adds exactly two ordered entries to five presets, conserving every prior entry. Counts become HappyPath120, Essential62, Ecosystem128, Opinionated178 and Nuxt179; Incremental0 and opt-in104 are unchanged. The eslint map changes only the two membership arrays. English/Japanese generated references and three other localized index tables change only those preset fields; complete existing examples and every other metadata cell remain conserved.

The source CLI observer records144 actual processes: six broad/default preset choices, empty Incremental, explicit-off, three Vue versions and original explicit51 selection for each full carrier. It persists raw status/signal/error/stdout/stderr before assertions and reads, source bytes/hash, entire JSON expectations/results, unchanged configuration and an exact-source build receipt. The existing tooling bootstrap/full differential artifacts own execution; there is no new CI stage. The editor test checks84 whole serialized vectors across implicit Ecosystem, recommended, off, Vue2/2.7/3 and Incremental, including Unicode/CRLF positions.

The original fifteen-carrier first-slice corpus and every n8n input, configuration, oracle and ownership file remain byte-exact. No runtime snapshot recapture, missing-fixture skip, selected-finding filter or unexplained output deletion is permitted.

## Delivery boundary

Source inspection, authored references and Rust1.99 formatting are preparation. Require exact-head Actions with actual new API/editor/CLI execution and existing complete n8n contracts, protected queue with unchanged instruction/performance limits, actual merge, real project acceptance, and a subsequent public installed release. Keep #8090 open until its release boundary is proven. This slice is outside the already frozen v0.440.0 source; urgent #8328 installed DOM/SSR/browser acceptance retains priority.

## Finite measurement admission and source-derived corrections

Initial source Actions `38028028229` refused the new test path before any400 measurement. The exact ten production/snapshot/test bodies are bound to actual original `e5702be6d0119a716499827fa7b918ce8c9dcbae` and reviewed `a7b5a205c44f25dee8f1105bc0d11077827b5f80` in `warm-type-backed-default-migration-manifest.json`, canonical SHA256 `c9afe6f838b54bbebacce0bc4ed1d476b57e5a1454eb341e4a61b8ab091acde2`. Nine whole preserved bodies cover all six tracked original warm400 corpus files, the first default corpus and both existing n8n configuration/case records. The finite helper rejects omitted, duplicated, unknown, missing or mutated bodies, including new owners whose before body must be absent. The actual400 workflow reads complete original/current Git bodies; unit controls bind every current source/corpus byte, both-side mutation refusals, a real temporary Git blob/absence/mode law and literal-cut isolation. Existing path/timing qualifications are composed unchanged; the workflow stays348 lines and records the new authority. This admits actual400 execution with original inputs and ceilings; it does not waive measurement or permit arbitrary future edits to those paths.

The source tooling census rejected the one new embedded petite-vue keyup spelling. Official natural-v-on generation adds exactly that row in the owned Patina shard and conserves every prior shard/row. This is derived source inventory, not regenerated runtime expected output.

Direct inspection of the unchanged editor collector establishes that `render_help(..., PlainText)` removes help backticks while preserving the diagnostic message. Correct only those authored LSP help fields; API help, messages, source carriers, ranges, existing style fix and all original corpora remain unchanged. Initial Rust source run `38028028313` subsequently reports the same numeric-longhand help discrepancy. No runtime vector is copied as an oracle or production producer altered to match a fixture. A fresh successor run is required.

Fresh main `042f24370b08031f0e281d0b7b5530ae8265b5e7` integration retains every incoming canonical byte and all350 lines, every reviewed production body and every original before/corpus body. Historical source `10c25afe31b4ef432a890bc85261af19e7c35af5` has successful actual400 run `38029414868` and all three n8n jobs in `38029414792`. Its tooling3 job `114147083270` correctly rejects an unavailable historical Git object in a shallow checkout. Unit proof now checks all current bytes without depending on repository history, independently exercises the strict reader in a real temporary Git repository, and preserves the actual400 original/current replay unchanged. No source, corpus, producer, qualifier, workflow fetch policy or performance ceiling changes. Fresh successor execution is required; historical success does not transfer.
