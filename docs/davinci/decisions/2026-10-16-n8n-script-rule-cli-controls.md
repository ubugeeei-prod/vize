# Declared whole-51 script controls and counterexamples

Tracking: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
The [authority inventory](./2026-10-16-n8n-full51-authority-inventory.md) retains
its audited `6740fcc7a60c39f6517bb11eb7d011fddb2b36ea` boundary. Adding provider
fixtures does not promote any identity in that inventory.

This slice declares sixteen script controls before native execution: the Cartesian
product of the four families below and `positive`, `clean`, `suppression`,
`noncode`. Case IDs are `script-<family>--<control>`.

| Family      | Explicit native identity                      | Oracle alias                               |
| ----------- | --------------------------------------------- | ------------------------------------------ |
| listeners   | `script/no-deprecated-dollar-listeners-api`   | `vue/no-deprecated-dollar-listeners-api`   |
| scopedslots | `script/no-deprecated-dollar-scopedslots-api` | `vue/no-deprecated-dollar-scopedslots-api` |
| computed    | `script/no-use-computed-property-like-method` | `vue/no-use-computed-property-like-method` |
| default     | `script/require-valid-default-prop`           | `vue/require-valid-default-prop`           |

Each positive has one selected finding; its paired clean, suppression and noncode
controls have no active findings. Each suppression retains one complete suppressed
provider finding. All calls use the unchanged explicit 51-error-rule projection,
its three original option blocks and official Vue `flat/base` infrastructure.
These four rules receive no options. Editor warning overrides and all six literal
scope patterns remain unchanged.

Six additional declared research cases preserve ordinary-object and template
counterexamples: listeners/scopedslots ordinary objects and template instances,
the computed template call, and an ordinary object containing a prop descriptor.
Both ordinary instance-property objects and the ordinary prop object are clean
under the pinned provider. The three template cases report their corresponding
selected rule. These cases remain part of the source execution scope even if
the sixteen script controls fail first. No finding or failure status is filtered.

The pinned ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and
TypeScript parser 8.65.0 were actually executed twice for every case. The fixtures
retain all 44 complete observations, including suppressed findings, foreign
fields, fix information, source coordinates, module/manifest hashes, configuration
and failure envelopes. Fixture tests reject changes to those complete packets and
pin the previous authored cohorts byte for byte.

The standalone `n8n-cli-config-script-rules-acceptance.mjs <output-directory>`
consumer requires the authenticated source build receipt and public configuration
package. It executes both cohorts, retains all provider and native responses,
then refuses any failed cohort with an aggregate error. The declared scope is
22 sources, 44 provider observations and 66 actual native CLI invocations
(baseline, repeat, outside cwd). No source CLI execution is claimed by the
provider recording or fixture tests. This manual campaign does not rerun the
1,369-file corpus and does not append unqualified cases to the older default
campaign.

The dispatch-only `.github/workflows/n8n-script-rule-controls.yml` becomes
registered after its reviewed main merge. It reuses one existing source producer
and authenticates the same-run artifact in one consumer. Ordinary PR checks do
not substitute for this later manual execution; a research disagreement must
leave that execution red with its complete artifact retained.

At preparation, stable source `163c349e00cc49d117da0a24b2d5acf26aa3c9b1` reports
whole member spans for the legacy instance properties, whereas the official
provider reports the property token. These are observable qualification gaps;
the fixture keeps exact independently authored target coordinates. Active
listeners product work belongs to safe-script parent #9222 and canonical-template
child #9251, tracked in #9217. Historical broad script receiver warnings also
differ from the ordinary-object provider controls. Neither a draft template
walker nor a green narrower campaign qualifies these counterexamples.

This test-only change makes no native API, installed package, exhaustive
four-rule, whole-51, editor, performance or release claim. Actual source receipts
must identify their exact immutable build. All n8n upstream access remains read-only.
