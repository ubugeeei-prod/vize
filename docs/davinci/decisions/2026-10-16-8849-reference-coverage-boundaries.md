# Preserve two findings outside the pinned ESLint reference's coverage

Issue: [#8849](https://github.com/ubugeeei-prod/vize/issues/8849). This is a bounded
reference-coverage classification, not completion of the parity/release gate.

The Matrix run on `d2506ca6b5e31252973f05981829c0367cacf26b` reported two
unmatched Patina findings under `ecosystem`. Full source and pinned reference
execution show that both findings implement the existing default rule policy.
Removing these diagnostics to match an unobservant reference would regress Vize.

## Source and visitor evidence

- GUI.for.SingBox revision `1be4816dea1e8793d8e08ae4f8f016c28101672d`,
  `frontend/src/components/_common/AppShell.vue:18:19–18:36`,
  `vue/v-slot-style`, warning. The authored `#="{ Component }"` is a default
  slot directive in Vue's compiler. Vue generates identical render code for
  `#`, `#default`, and `v-slot` here. The rule's existing component default
  requires `v-slot`. `vue-eslint-parser@10.4.1` instead represents bare `#`
  as a non-directive attribute with a `VIdentifier` key. The reference rule's
  `VAttribute[directive=true][key.name.name='slot']` visitor never sees it.
  Replacing `#` with `#default` makes the reference report the same style policy.
- mobile-web-best-practice revision `34a793b13492405f75c97b46253df5ee142014cc`,
  `src/views/home/widgets/card.vue:10:25–10:39`, `vue/no-mutating-props`, error.
  `notebook` is declared with class `@Prop` and `v-model="notebook.notes"`
  writes it. Patina's existing prop analysis recognizes this declaration.
  The reference's `defineScriptSetupVisitor.onDefinePropsEnter` and
  `defineVueVisitor.onVueObjectEnter` collect script setup and object Options
  API props, but not this class decorator. Declaring the same prop through
  the Options API makes the reference report the mutation.

The sources, complete original Matrix divergence packets (losslessly gzipped),
original native linter packets, source hashes, complete finding identities,
revisions and runtime pins live in
[`reference-coverage-8849`](../../../tests/_fixtures/differential/linter/reference-coverage-8849/contract.json).
`oracle-source-evidence.json` records hashes and published tarball locations of
the exact reference parser and rule implementations. Tests execute those real
packages, verify their hashes and AST shape, and compare Vue-generated code.
No upstream repository is modified or contacted with comments.

## Bounded classification

Reuse the existing `patinaOutsideBaselineSurface` bucket. A reviewed row gains
`referenceCoverage.category: "reference-coverage-gap"`, with its reason,
tracking issue, source hash, revision, runtime versions, options and retained
packet hash. Every original finding field remains present. Markdown reports
show this category separately from false positives and false negatives.

A row matches only the reviewed project and revision, whole-source SHA-256,
complete rule/file/span/severity/message tuple, `ecosystem` preset, severity-only
rule configuration (default options), and all four pinned reference runtimes:
ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1, and
@typescript-eslint/parser 8.65.0. Both rule option lists are empty; in particular,
`no-mutating-props` retains its default `shallowOnly: false` behavior.

Only one occurrence of each reviewed row can enter this category. If the
reference reports any finding for that rule in that file, classification is
refused, even if its location differs. Changed source, revision, options,
severity, range, message or runtime restores ordinary scoring. The source
hash and reference scan are evaluated only after a complete reviewed row
matches; unrelated findings do not scan the reference packet.

Eligibility is checked independently of received native findings, using the
same whole-source/runtime/configuration contract and the measured input list.
Each eligible case must have exactly one complete expected native finding.
Missing or duplicated expected diagnostics are retained in
`referenceCoverageExpectations` and make the report unusable, so losing a
correct Vize warning cannot turn the comparison green. Changed source, runtime,
revision, preset or options creates no expectation and keeps ordinary scoring.

The native report now retains the complete original Vize result envelope and
ESLint result objects under `raw`, before normalization and classification.
Historical packets remain immutable. No rule is disabled, no location is
rewritten, and neither zero false-positive nor zero false-negative budget rises.
The source-free JavaScript comparator does not apply these source-bound cases.

## Executable controls

`tests/tooling/lint-reference-coverage.test.mjs` checks frozen full packets,
source identities, pinned package/visitor hashes, the two missing reference
findings, the supported `#default` and object-prop positive controls, and valid
`v-slot`/read-only negative neighbors. It executes the actual Rust matcher and
actual report runner. Appending a newline to either full source restores the
original finding to false positives while retaining both raw packets. Deleting
either expected row fails enforced report integrity; duplicating it remains
visible and unusable. With a changed source hash, neither case is eligible,
including when its native row is absent.

Rust matcher mutations cover every finding identity field, each runtime pin,
revision, preset, explicit options/severity changes and newly visible reference
coverage, for both cases. Product regression tests separately assert that Vize
continues warning on bare component `#` and reporting the class prop mutation,
while retaining valid template slots, writable class locals and read-only props.

Exact-head Actions must pass before this classification is accepted. This
change does not qualify a full Matrix campaign or close #8849.
