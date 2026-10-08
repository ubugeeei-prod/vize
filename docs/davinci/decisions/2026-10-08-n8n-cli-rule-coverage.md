# Latest n8n CLI rule evidence inventory

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
This inventory follows the read-only CLI requirement `5c2a2cf3` and the
[explicit owned 51-rule projection](../../../tests/_fixtures/n8n-cli-adoption.json).
It reviews committed independent packets at source `18217af7`; it does not
declare all51 native correctness, full monorepo or installed-release parity.

**W** means a complete independent positive/clean packet pair in the
[owned CLI corpus](../../../tests/_fixtures/differential/lint/n8n-cli-config/cases.json).
Every call enables all51 explicit identities with actual ESLint10.4.1,
eslint-plugin-vue10.9.2, Vue parser10.4.1 and TypeScript parser8.65.0.
Public module/package hashes, full findings, fixes and envelope fields are
retained. These five rules still have bounded witnesses, not exhaustive semantics.

**T1** is the live provider check in
[no-unused-vars oracle](../../../tests/tooling/patina-no-unused-vars-oracle.test.ts);
it filters findings and keeps message/range projections, not whole envelopes.
**T2** is the offline provider-attributed tuple corpus in
[multiple-root tests](../../../crates/vize_patina/src/rules/vue/no_multiple_template_root/tests.rs).
**U** locates only native unit/snapshot evidence in the rule modules.
No complete independent pair was located for the 46 T/U rows.

The last column is raw baseline/effective finding count from the licensed
master source replay in [run37733007223](https://github.com/ubugeeei-prod/vize/actions/runs/37733007223).
That authenticated source at `18217af7` retained all1369 inputs/nine roots/19
scriptless files and passed the whole configuration transformations before
rejecting an authored expectation missing static help metadata. Its full
artifact is `11530423750`, SHA256
`775f20a9c57c728832cbd94768166ec3e887277483ae672ca877d18e85f750c0`.
A zero is an observation, never false-negative or semantic-accuracy credit.

|   # | Vize identity                                 | Official identity                          | Independent authority                                         | Observed baseline/effective |
| --: | --------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------- | --------------------------: |
|   1 | `vue/attribute-hyphenation`                   | Same                                       | W: camel/hyphen attribute; never inversion                    |                     18 / 18 |
|   2 | `vue/component-name-in-template-casing`       | Same                                       | W: registered kebab/Pascal import; kebab inversion            |                       0 / 0 |
|   3 | `vue/no-child-content`                        | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   4 | `vue/no-deprecated-functional-template`       | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   5 | `vue/no-deprecated-html-element-is`           | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   6 | `vue/no-deprecated-inline-template`           | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   7 | `vue/no-deprecated-router-link-tag-prop`      | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   8 | `vue/no-deprecated-scope-attribute`           | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|   9 | `vue/no-deprecated-slot-attribute`            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  10 | `vue/no-deprecated-slot-scope-attribute`      | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  11 | `vue/no-deprecated-v-bind-sync`               | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  12 | `vue/no-deprecated-v-on-native-modifier`      | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  13 | `vue/no-deprecated-v-on-number-modifiers`     | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  14 | `vue/no-dupe-v-else-if`                       | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  15 | `vue/no-duplicate-attributes`                 | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  16 | `vue/no-multiple-template-root`               | Same                                       | T2: offline translated provider tuples                        |                       5 / 0 |
|  17 | `vue/no-template-key`                         | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  18 | `vue/no-textarea-mustache`                    | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  19 | `vue/no-unused-components`                    | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  20 | `vue/no-unused-vars`                          | Same                                       | T1: filtered message/range provider projection                |                       0 / 0 |
|  21 | `vue/no-use-v-if-with-v-for`                  | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  22 | `vue/no-useless-template-attributes`          | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  23 | `vue/no-v-for-template-key-on-child`          | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  24 | `vue/no-v-html`                               | Same                                       | W: scripted and scriptless positive/clean                     |                       0 / 0 |
|  25 | `vue/require-component-is`                    | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  26 | `vue/require-component-registration`          | `vue/no-undef-components`                  | U: native unit/snapshot only                                  |                       0 / 0 |
|  27 | `vue/require-toggle-inside-transition`        | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  28 | `vue/require-v-for-key`                       | Same                                       | W: missing key / keyed loop                                   |                       1 / 0 |
|  29 | `vue/sfc-element-order`                       | `vue/block-order`                          | W: template-before-script / script-before-template; inversion |                       0 / 0 |
|  30 | `vue/use-v-on-exact`                          | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  31 | `vue/v-slot-style`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  32 | `vue/valid-attribute-name`                    | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  33 | `vue/valid-template-root`                     | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  34 | `vue/valid-v-bind`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  35 | `vue/valid-v-cloak`                           | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  36 | `vue/valid-v-else`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  37 | `vue/valid-v-for`                             | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  38 | `vue/valid-v-html`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  39 | `vue/valid-v-if`                              | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  40 | `vue/valid-v-memo`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  41 | `vue/valid-v-model`                           | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  42 | `vue/valid-v-on`                              | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  43 | `vue/valid-v-once`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  44 | `vue/valid-v-show`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  45 | `vue/valid-v-slot`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  46 | `vue/valid-v-text`                            | Same                                       | U: native unit/snapshot only                                  |                       0 / 0 |
|  47 | `script/no-deprecated-dollar-listeners-api`   | `vue/no-deprecated-dollar-listeners-api`   | U: native unit/snapshot only                                  |                       0 / 0 |
|  48 | `script/no-deprecated-dollar-scopedslots-api` | `vue/no-deprecated-dollar-scopedslots-api` | U: native unit/snapshot only                                  |                       0 / 0 |
|  49 | `script/no-ref-as-operand`                    | `vue/no-ref-as-operand`                    | U: native unit/snapshot only                                  |                       0 / 0 |
|  50 | `script/no-use-computed-property-like-method` | `vue/no-use-computed-property-like-method` | U: native unit/snapshot only                                  |                       0 / 0 |
|  51 | `script/require-valid-default-prop`           | `vue/require-valid-default-prop`           | U: native unit/snapshot only                                  |                       0 / 0 |

The generic divergence matrix cannot qualify this contract: it selects by
preset/status and filters foreign configuration findings. Its generic map
marks no-undef-components unimplemented. Eight selected identities have
empty preset lists: functional-template, html-element-is, inline-template,
router-link-tag-prop, v-on-number-modifiers, multiple-template-root and the
two deprecated dollar APIs. The explicit projection retains every identity.

A separate actual `db6f2c09` source-Linter campaign with 26 full51 authored
inputs reproduced the next concrete gaps, with complete repeated provider
and native packets and an authenticated probe digest:

- [#8275](https://github.com/ubugeeei-prod/vize/issues/8275): ref factory identity loses Vue aliases/namespace calls and falsely accepts unrelated local/package/shadowed factories. Paired `.value` and direct Vue imports control both directions.
- Registration misses lowercase and x-/ion-/router-/nuxt-style unknown components compared with no-undef-components. This differs from [#8273](https://github.com/ubugeeei-prod/vize/pull/8273), which fixes registered-only casing.
- valid-v-slot misses modifiers, a dynamic argument using its same slot parameter, and a valueless component default slot.
- no-child-content misses comments overwritten by v-html.

These research packets add named defect evidence, not whole-rule admission.
The focused ref-identity owner records its original corpus and other gaps in
the #8275 companion record; registration, slot and child-content corrections
remain separate slices. Retired dollar API template/object ownership also
needs a source producer check; provider/source inspection alone is unfinished.
