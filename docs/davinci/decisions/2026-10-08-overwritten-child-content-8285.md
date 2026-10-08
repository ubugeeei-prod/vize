# Overwritten comment child content (#8285)

Refs [#8285](https://github.com/ubugeeei-prod/vize/issues/8285) and the open
[n8n acceptance umbrella #8142](https://github.com/ubugeeei-prod/vize/issues/8142).
The current n8n contract explicitly enables `vue/no-child-content` as an error.
Upstream n8n remains strictly read-only. Every executable source in this slice
is independently authored; the latest non-master adoption source is neither
vendored nor executed.

## Decision and history

`v-html` and `v-text` overwrite comment children as well as text, elements and
interpolations. The existing native rule has excluded
`TemplateChildNode::Comment` since its introduction in #42; subsequent history
only moved/refactored that code. There is no comment exception test or recorded
policy in that history. This correction deliberately changes that omission:
it counts already parsed comment children in the same existing child match.
No source scan, parser pass, pipeline stage, allocation or option is added.

Empty and whitespace-only child content remains valid. Comments outside the
content-directive element and comment-looking attribute strings remain valid.
Text, element and interpolation positives retain their existing behavior.
The native opening-tag location, message/help and absent fix are preserved;
this slice does not adopt ESLint's child-range/removal-suggestion API.

## Owned controls and independent authority

The [differential corpus](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/controls.json)
contains 36 complete authored SFCs. Both `v-html` and `v-text` are exercised with
and without a script, each with plain/empty/whitespace-surrounded comments,
empty/whitespace content, text, an element and interpolation. Four controls
retain ordinary/outside comments and a comment-looking attribute string.

The [owned configuration projection](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/config.json)
retains all 51 literal n8n rule identities and three options from requirement
revision `5c2a2cf3d837b6538a3330a4d7cc817e4b6f04e2`. The functional projection is
not an executed upstream settings package. Seven explicit independent rule
aliases and the three options are preserved in the provider setup.

The actual independent provider is ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1 and TypeScript parser 8.65.0, with the pinned official
Vue base processor `vue/vue`, `vue/comment-directive` and `vue/jsx-uses-vars`.
[Provider entry hashes](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/independent/providers.json)
are checked before execution. Each complete packet runs twice; only the
absolute authored `filePath` is converted to the same owned relative filename.
Messages, foreign findings, severities, suggestions, fix ranges, suppressions,
fatal counts, deprecated-rule metadata and original source stay whole.
All 72 raw observations, including any provider errors, are retained before
assertions in `target/tooling/overwritten-child-content-8285/whole-provider.json`.

The 36 whole native API packets and 36 public JSON report packets use the
same full51 projection. They remain separate from the independent provider's
whole packets. Foreign `vue/no-v-html` findings are retained, not filtered.
The [Rust API test](../../../crates/vize_patina/tests/overwritten_child_content.rs)
checks every diagnostic field and complete public report twice; its parser
control proves the original comment node already exists.

## Actual source evidence

The source-before baseline is `68e0dc38973a8690bac8ce1c0d6b6349eb4a491a`.
The declared Rust 1.99.0 compiler was used directly, with a fresh private
worktree target `target/no-child-content-source`; no Cargo target was imported.
The [observer receipt](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/source-before/source-before-receipt.json)
binds the original producer, full captured output and compiler. Its observer
binary SHA-256 is `cc9b3ec307b7ae5a2a74e655524bcfb792a7144558f2cc73487919c215923d1f`.
The original raw36 native packet SHA-256 is
`3d17eaf10ee8b2c47e8c08844d8610e382e4d2481363a29380be648988f7f15a`.

The [actual regression-before failure](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/source-before/regression-before.log)
names all12 missed comment controls; it was not a stale binary or inferred
source result. All24 noncomment controls retain their entire before/after
packets. The [complete before packets](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/source-before/)
and [original capture observer](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/source-before/observer.rs.txt)
remain immutable. The current expected policy is separately captured in
[native packets](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/native/).
A whole-delta test verifies the only twelve changes add one comment finding
and update the corresponding error counts; every other field remains equal.

After the one-branch correction, the private source build passes all three
Rust API tests, all seven existing rule tests and both independent replay/delta
tests, with zero skips. The official consumer-migration inventory check passes
after adding only its three new test-source rows.
The dirty-owned-tree [after receipt](../../../tests/_fixtures/differential/lint/overwritten-child-content-8285/native/producer.json)
records observer SHA-256 `9182ce60a04ac4e91b8d6b59228027821bc53484916aa69bc8c8e4f7f6f341cd`;
it is bounded local source evidence, pending exact-head hosted qualification.

To reproduce the regression-before, use an isolated checkout of the baseline,
apply only the new corpus and Rust test, and run that test's
`actual_source_api_reports_overwritten_comments_and_preserves_negative_controls`
with its own private target. The preserved observer can be copied to that
checkout's `crates/vize_patina/examples/overwritten_child_probe.rs` and run via
`cargo run -p vize_patina --profile ci --example overwritten_child_probe -- <output>`
with declared Rust 1.99.0 and `CARGO_TARGET_DIR=target/no-child-content-source`.

## Delivery and limits

Exact-head Actions, protected candidate checks and actual signed merge remain
pending. Queue admission and release verification belong to the coordinating
maintainer lane; an open or source-green PR is not completed delivery.
This slice leaves #8142 open. It does not qualify all51 rules, actual installed
CLI/package artifacts, upstream monorepo accuracy, comment-disable policy or
comparative performance. The existing acceptance projection and scoped entries
are unchanged; no inherited fixture or public contract is silently replaced.
