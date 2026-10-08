# Report bound deprecated slot attributes

Decision for [#8289](https://github.com/ubugeeei-prod/vize/issues/8289),
Refs [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

Report static-argument `v-bind:slot` bindings through the existing parsed
directive callback. This covers `:slot`, `v-bind:slot`, same-name shorthand,
dot shorthand `.slot`, and modifiers without adding a parser, stage, source scan,
or child allocation. The authored key ends at the maximum of the parsed argument
and final modifier locations, retaining synthesized prefix-only modifiers. Static
`slot="name"` retains its exact existing location, message, help, severity,
labels, and no-fix behavior. The new findings retain that native metadata and
point to the whole directive key. Migration fixes remain a separate feature.

Computed arguments, including `:['slot']`, object bindings, modern `v-slot`,
and unrelated attributes remain outside the new detection. Vue 2 retains its
legacy syntax. Template disable comments remain honored. The legacy Options API
fixture retains its complete existing registration finding; it is not relabeled
clean. Global/registration/casing, deprecated HTML `is`, and numeric-modifier
policies are separate work and are not changed here.

## Fresh source evidence

The actual baseline is fresh-main
`26e56ac6a0de3f9f838db55bbdb71757317f2435`, tree
`8b546d7e16e70dc11b4b5b50758b44c92e695837`. Declared Rust 1.99.0 built the
source API observer in a new private `target/bound-slot-source`; no Cargo target
was imported. All 10,186 indexed production/manifest/config entries remained
unchanged through the baseline build and original 40-case execution. The same
authenticated retained baseline binary then executes all 46 owned sources.
The complete scripted
`:slot="name"` result is empty before the fix. The exact same owned source yields
one independent deprecated-slot error and its full fix.

The [46-case source corpus](../../../tests/_fixtures/differential/lint/bound-slot-attribute/cases.json)
contains 23 variants in both scripted and scriptless forms. Every whole native
packet repeats exactly. The final producer adds one named diagnostic/error to
22 cases; all other 24 whole packets remain unchanged. A named delta check proves
every preexisting diagnostic and field remains unchanged in the 22 additions.
The actual regression suite compiled against the byte-exact original rule fails
both new regression checks in the original 40 corpus (all 16 full-packet cases,
each twice, and the key invariant); the Vue 2 control passes. The extended
46-case source suite passes all three checks after the final producer repair.

The complete independent corpus uses ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1, and @typescript-eslint/parser 8.65.0. Public entrypoint
and package manifest hashes are checked. Every call enables the full explicit
51-rule/three-option projection plus official Vue base processor `vue/vue` and
its comment-directive / JSX infrastructure. All 92 complete responses repeat
exactly, including foreign diagnostics, suppressions, suggestions/fixes, source,
and envelope counts. Raw responses and provider errors are saved before any
assertion. Only a named exact-filePath recording step changes provider paths.

The first producer attempt exposed an incorrect prefix-only key location:
`raw_name` on these parsed callbacks contains `:` / `v-bind`. The independently
authored key invariant rejected it. The retained first packets are historical;
the final span uses existing parsed argument/modifier positions instead. An
independent peer then found `.slot` has a synthesized `prop` modifier located
only at `.`. Six dot-shorthand controls retain the original 40 cases and 80
provider packets unchanged; the actual prior observer and compiled regression
both reject the two `.slot` key spans. Their complete failed packets are retained.
Taking the maximum parsed argument/modifier end repairs this boundary. No
oracle field or finding was removed to make that attempt pass.

| Source receipt                          | SHA256                                                             |
| --------------------------------------- | ------------------------------------------------------------------ |
| Baseline observer                       | `4172bcb0657b5f029ccf6cc80a6faf57e57e0fbcb1c1cda393236cfa8025929a` |
| Baseline complete 46-case raw packets   | `24df0ac6d0cfb0f8e2fda2861917d54919b1b69635c91742813679cd3f9c4833` |
| Final working-tree observer             | `2c50597b0012450ca3ba37ec3e7c29caf33bbed639028f70497f75c4b917175f` |
| Final working-tree complete raw packets | `4690666b8031f0eea914a308ef5b2491ec17b8ed42a95b7468afc15f5580ef41` |
| Complete independent raw packets        | `54ccb5004f6b41debf9b8db38e3a3e8c814829f7d8b7e12a5f8019c70d9a6243` |

These baseline/working-tree receipts do not transfer to a future committed head.
Exact-head Actions and protected queue delivery must build and run the current
source again. Existing consumer inventory and bilingual generated rule references
must remain fresh; source and fixture fields do not bypass their checks.

The hosted source suite also rejected stale CLI `explain` examples. Only the
owned slot page gains the documented bound example in each of the three existing
locales; all other 398 pages remain byte-exact per locale. The whole actual EN
slot page matches the retained hosted output. The complete actual EN assertion
left/right differs only in slot; both executed strings already have the imported
casing example. An earlier two-page comparison used the older declared PR-head
snapshot instead of the executed right packet and is explicitly corrected.
Fresh signed main already owns casing freshness. JA/ZH runtime output was not
observed after the first EN assertion failed.

## Retained wider research

The [historical full-packet research](../../../tests/_fixtures/differential/lint/deprecated-template-research-47c6/README.md)
preserves all 118 owned sources, 236 whole provider responses, and 236 whole
native observations at committed source
`47c6f998d49cd6c7b790550098b3378639b949f7`. It includes the retained observer,
physical binary hash, build/execution receipts, complete identical 10,178-file
source inventory, provider pins, and classifications. That source API custody is
historical research and grants no proof for this fresh source or later heads.

| Deprecated rule       | Cases | Provider positive / target-clean | Historical native positive / target-clean |
| --------------------- | ----: | -------------------------------: | ----------------------------------------: |
| functional-template   |    10 |                            4 / 6 |                                     4 / 6 |
| html-element-is       |    16 |                            8 / 8 |                                     8 / 8 |
| inline-template       |     8 |                            4 / 4 |                                     4 / 4 |
| router-link-tag-prop  |    12 |                            6 / 6 |                                     6 / 6 |
| scope-attribute       |     8 |                            2 / 6 |                                     2 / 6 |
| slot-attribute        |    10 |                            8 / 2 |                                     6 / 4 |
| slot-scope-attribute  |     8 |                            4 / 4 |                                     4 / 4 |
| v-bind-sync           |    16 |                           10 / 6 |                                    10 / 6 |
| v-on-native-modifier  |    12 |                            6 / 6 |                                     6 / 6 |
| v-on-number-modifiers |    18 |                           10 / 8 |                                    14 / 4 |

This table counts target presence only. Whole-packet parity and exhaustive rule
accuracy remain unqualified. The retained research names custom-tag/bound HTML
`is` and numeric first-modifier/digit/integer-prefix differences, 26 jointly
positive span differences, 22 absent native fixes, and 15 foreign identity or
cardinality differences. All are separate TODOs requiring their own fresh source
and policy evidence. Historical native tests explicitly retain some of these
policies; do not silently rewrite them or call them fixed from an unrelated PR.

## Delivery limits

This closes only the named bound-slot source defect after actual protected
delivery. It leaves #8142 and its full51, current CLI/installed public package,
unlicensed upstream package identity, whole monorepo/checker/editor, and
performance acceptance open. The latest upstream branch remains read-only
requirement evidence; no source was copied, executed, or changed upstream. The
existing six literal adoption overrides and suppression policy remain unchanged.
Root owns queue admission, signed merge, and public release qualification.
