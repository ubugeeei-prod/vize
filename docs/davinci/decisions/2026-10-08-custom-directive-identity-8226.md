# Custom-directive identity regression (#8226)

The strict full Real Project Matrix run [37700140534](https://github.com/ubugeeei-prod/vize/actions/runs/37700140534), source `fd6241bf8ea5466794cc955138a59a9b75a8aac2`, exposed two real `vue/no-duplicate-attributes` false positives in the pinned ant-design-vue fixture at `7483836f0adac76516e527df893c3d84a7cd4005`. `DemoBox.vue` and `CopyableIcon.vue` bind both `v-clipboard:copy` and `v-clipboard:success`; the old fallback compared only `v-clipboard`.

## Decision

[#8226](https://github.com/ubugeeei-prod/vize/issues/8226) keeps custom directive identities in a separate namespace, using the directive name, static argument and ordered modifiers. The same complete identity still reports a duplicate. A dynamic argument cannot establish equality, matching the existing conservative argument handling. A custom directive cannot collide with a literal bound attribute that happens to contain its name.

The existing `v-bind`, `v-on`, `v-model` branches and built-in fallback retain their original keys, messages, spans and options. In particular, the complete historical [#2376](https://github.com/ubugeeei-prod/vize/issues/2376) static/bound pair, [#2399](https://github.com/ubugeeei-prod/vize/issues/2399) dynamic binding and [#7007](https://github.com/ubugeeei-prod/vize/issues/7007) duplicate `id`/`v-if` contracts remain enforced. Official eslint-plugin-vue 10.9.2 ignores all non-bind directives more broadly; copying that skip would discard Vize's existing event/model/built-in controls and is outside this fix.

## Corpus and qualification

`tests/_fixtures/differential/linter/custom-directive-identity-8226` retains both complete original sources as `.vue.txt`, with authenticated pinned Git blobs, byte lengths and SHA-256 hashes. It preserves the complete original Matrix file reports and complete expected CLI reports. Only the two false duplicate findings are removed; all 13 unrelated warnings, locations, help and metadata remain. The ecosystem/no-config JSON CLI command runs twice per file and compares each complete output packet, exit status and stderr, while checking that inputs stay byte exact.

The public rule API compares complete diagnostic packets for 32 intended scenarios, twice with fresh state. These cover real duplicates, distinct arguments/modifiers, ordered modifier identities, dynamic arguments, Unicode spans, custom/bound namespaces, coexistence options and the historical event/model/built-in controls. The public `lint_template` result includes the existing literal-duplicate parser warning at the second `id` name; every rule diagnostic remains intact. No parser fields are filtered. The old snapshots and unrelated corpus expectations are unchanged. Independent read-only semantic review passed; exact-head hosted compilation/runtime and protected qualification are pending. No heavy local Rust build is required.

## Remaining work

The other strict Matrix findings remain unresolved and separate: ant-design-vue conditional-template key traversal and attribute-inheritance scope/location, uploader reserved keys/prop mutation, mobile transition/prop mutation, and unusable upstream range/parser packets. This two-finding fix does not claim full project or Matrix parity. Old caps, budgets, oracles, classifications and source contracts are unchanged. Native replacement/fix-history and installed release verification remain unfinished. No upstream state is changed.
