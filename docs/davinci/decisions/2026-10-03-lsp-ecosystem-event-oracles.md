# Registry-pinned component event completion oracles

## Decision and source evidence

Real Project Matrix run 37098199031 executed source
`ae874862fb175f1e3a6df27cbd412b270f664d60`. Six authored component
completion oracles failed exact item-count checks. Their entire logged label
arrays retain every previous label at the same rank, followed only by the
component's declared events. These are complete arrays printed by the original
assertion, not retained raw JSON-RPC envelopes.

The established production behavior comes from
[`fix(lsp): complete declared component events` (#7090)](https://github.com/ubugeeei-prod/vize/pull/7090).
Its imported component projection reads genuine Croquis alpha `EmitContract`
facets and publishes event items at empty or `@` attribute prefixes. The old
registry ranks omitted those events. The native Vue 2 changes in the failed
Matrix source did not change these legacy product or oracle paths.

Refresh only the following literal counts and append these exact ranked labels:

| Project        | Previous count | Current count | Appended labels and ranks           |
| -------------- | -------------: | ------------: | ----------------------------------- |
| vue-vben-admin |             30 |            31 | `@update:modelValue` at 30          |
| pinia          |             31 |            32 | `@update:modelValue` at 31          |
| varlet         |             30 |            31 | `@update:theme` at 30               |
| element-plus   |             30 |            31 | `@expand` at 30                     |
| vue-datepicker |             32 |            34 | `@activate` at 32, `@set-ref` at 33 |
| misskey        |             37 |            38 | `@closed` at 37                     |

Each source was fetched at the registry's existing revision. Independently
recomputing `SHA1("blob " + byteLength + NUL + originalUTF8Bytes)` matches the
returned Git blob. The new reduced corpus preserves each event declaration
byte for byte and records the whole upstream file's blob, path and revision.
Its reduced inputs do not claim the whole upstream file's blob identity.

| Project        | Pinned source                                                                                                                                                                 | Whole-file blob                            |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| vue-vben-admin | [theme-button.vue](https://github.com/vbenjs/vue-vben-admin/blob/748b182e440863711dc9624924e92440f5ddc4d3/packages/effects/layouts/src/widgets/theme-toggle/theme-button.vue) | `178f9776564a8a2f1833e778332296eabe8a5f11` |
| pinia          | [VersionSelect.vue](https://github.com/vuejs/pinia/blob/59ec7da3f0130caa66fae6541557cf3c5b10f78b/packages/online-playground/src/VersionSelect.vue)                            | `2232594c0d503ddd48f67dde049bc1519b06ef15` |
| varlet         | [Header.vue](https://github.com/varletjs/varlet/blob/1007097bde3f6c06326e0057ac6a7cf4ab7ff519/packages/varlet-ui-playground/src/Header.vue)                                   | `93f620186e452e99ad4b6f223beebbeda667633c` |
| element-plus   | [node.vue](https://github.com/element-plus/element-plus/blob/20298a21ae54a5417955373b80130ae7c548a6bb/packages/components/cascader-panel/src/node.vue)                        | `f73ce1ac49870f4ee737dc27f6de942d3bd81d5d` |
| vue-datepicker | [ArrowBtn.vue](https://github.com/Vuepic/vue-datepicker/blob/f1a5c11028885e02e81ed379ff04107a7d99d964/packages/lib/src/VueDatePicker/components/Common/ArrowBtn.vue)          | `ed2f36efbf9bacbc54c3da3cd1c869a54c3c700b` |
| misskey        | [MkTooltip.vue](https://github.com/misskey-dev/misskey/blob/810faa8e5db4bbf9d6d408a61df148559340ee99/packages/frontend/src/components/MkTooltip.vue)                          | `08a3f02f6578a17d567ba704270e1ce2871303cc` |

## Regression gates and unfinished work

The existing oracle still checks exact counts, every label's rank, unsaved
dependency changes, restoration equality, versioned diagnostics and file
lifecycle responses. New positive and negative helper laws accept a complete
event list and reject a missing event, an undeclared extra event and an event
at the wrong rank. Registry laws bind the six updated tails to their original
fixture revisions and component files.

An actual imported-component completion regression consumes all six reduced
script inputs through the existing `ServerState`, descriptor and component
surface route. A separate negative input keeps event-like strings and comments
out of the event completions. These Rust laws require fresh source-built
Actions; local helper success is not product execution evidence.

TODO: run the fresh exact-head full source Check, required PR checks and protected
merge queue, then verify actual merge. Rerun the required Real Project Matrix
against the accepted merged source. Other Matrix failures remain failures.
The separate PrimeVue, PrimeVue Volt and Buefy hover differences require genuine
source and dependency/type contracts before changing any hover golden. In
particular, `string[]` becoming `any` is not accepted by this change.

This fixture correction changes no product implementation, fixture revisions,
budgets, parser/provider source, default routing or assertion strength. Native
completion, complete response/history acceptance and #6883 closure remain
unfinished. The paired decision is recorded on #3952 and #6883.
