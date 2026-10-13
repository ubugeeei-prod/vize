# Vue lint baseline processor

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Add only `processor: "vue/vue"` to the actual Rust fixture runner's embedded
ESLint flat configuration and its retained JavaScript compatibility entry
point. Keep the exact caller-selected rules and severities; do not spread the
public base rules into a different preset/default vector. Preserve all projects,
skips, counts, budgets, parser options and classification/refusal policy.

## Authenticated omission

Original Matrix [38205026873](https://github.com/ubugeeei-prod/vize/actions/runs/38205026873),
attempt1, source `ef34d1a0409fc711a6c5cefcf0708b0cce78f502`, failed the lint
divergence gate in original project3/job114666509135. Its actual producer at
`tools/commands/fixtures/lint-divergence-report.rs` has no processor in the
embedded Vue-parser/TypeScript-parser flat configuration. The whole68,493-byte
producer has SHA256
`70eb083efe07d0eaf417c7b4d6f6a3d944d53738662ec2740f6ba684dc43ebe7`;
the reviewed implementation base `5c54b1195fa8c0bf151e0a2862bf4238a3597b4e`
retains those exact bytes. The correction adds no line to the1976-row file.

The pinned public providers are ESLint10.4.1, eslint-plugin-vue10.9.2,
vue-eslint-parser10.4.1 and @typescript-eslint/parser8.65.0. The signed plugin
v10.9.2 tag identifies commit `9aa463a4aa4ad3c37d7facacda2e40ad7880ea92`.
Its [public flat base](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/configs/flat/base.ts)
selects `vue/vue`. The [public processor](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/processor.ts)
passes through source and applies/removes comment-directive controls; the
[rule documentation](https://eslint.vuejs.org/rules/comment-directive.html)
explains why the rule's intermediate control reports require postprocessing.

Original artifact11727196356 retains2,807 Ant Design Vue `clear` control
findings. Four belong to the complete original1350-byte ThemeIcon source at
revision `7483836f0adac76516e527df893c3d84a7cd4005`, retained unchanged in
the lossless `tests/tooling/fixtures/lint-baseline-processor/ThemeIcon.json`
UTF8 source carrier with SHA256
`eb0f821769f5aa61f9819562c9420c411cf755240f2f33564b7483611f38423b`.
This proves leaked controls, not the cause of every original unusable result.

## Complete producer and raw boundary laws

The cheap existing tooling campaign runs
`tests/tooling/lint-divergence-processor.test.mjs`. It executes the actual
extracted Rust embedded JavaScript through its original stdin/stdout protocol
and the real compatibility function with the pinned installed providers.
Compare each whole JSON result to the public processor with the same original
parser configuration and selected rules. JSON transport serializes public
getter values and omits undefined fields identically; no diagnostic field,
source, suppression packet or deprecation metadata is cropped.

Controls cover unsuppressed warnings, next-line/same-line disable, rule-scoped
and all-rule disable/enable, another visible rule inside a selectively disabled
region, and the whole original ThemeIcon. The unchanged default selection does
not gain comment-directive/jsx rules. A real malformed TypeScript source retains
its complete fatal `ruleId:null` packet before collection and increments the
parse count. An explicitly mutated real warning with line0 remains unchanged
through the public processor and producer retention boundary, then increments
the invalid-range count. Both actual fatal and explicitly malformed-range packets
retain the unchanged zero-budget refusal. The optional test recorder retains
whole inputs, raw stdout and complete result packets before assertions; it adds
no production artifact or schema. The mutation does not reconstruct an original defect.

The Rust `collect_baseline_findings` boundary still counts null rule IDs and
invalid ranges before excluding those rows. `baseline_health_issue` still makes
either counter unusable. No classifier, budget or artifact schema changes.
Original bootstrap-vue4/admin2 invalid ranges and gui-for-singbox1 parse error
have no recoverable individual raw messages in the original artifact: the
producer discarded them before report serialization. Their causes and repair
remain unknown. If fresh Matrix evidence still needs those raw rows, propose a
separate additive custody change at the actual producer boundary; never filter
them, change the counts or turn an unusable comparison into parity.

## Delivery

The original seven laws genuinely produced3 passes/four failures without the
processor, then allseven passed after only the two property additions. The
final ten-case campaign produces three passes/seven failures with the original
configurations, then ten passes with zero skips after those exact two additions
under Node24.14.0. Type-aware lint has zero warnings locally; fresh hosted checks
remain required. No local Cargo builder, archive or dependency install
supplies qualification credit. Require independent source/boundary review,
fresh exact-head Actions and the full protected five-gate queue through actual
merge. Release ownership remains with the release lane; fresh complete Matrix
and installed acceptance are separate obligations. No release unblocking,
original-six cause or whole real-project parity claim follows from local laws.
