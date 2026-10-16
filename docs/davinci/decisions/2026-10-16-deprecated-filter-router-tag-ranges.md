# Filter expression and router tag report ranges

Issue: [#9218](https://github.com/ubugeeei-prod/vize/issues/9218).

The enforced Real Project Matrix found 14 location-only false-positive / false-negative
pairs in `PanJiaChen/vue-element-admin` at fixture pin
`6858a9ad67483025f6a9432a926beb9327037be3`: 13 filter interpolations and one
`router-link` tag attribute. Bound filter values already used the expression range.

For `{{ message | capitalize }}`, `vue/no-deprecated-filter` now reports
`message | capitalize`, excluding the braces and surrounding whitespace. For
`<router-link tag="button">`, `vue/no-deprecated-router-link-tag-prop` reports
the attribute name `tag`. For `:tag="element"` and `v-bind:tag.prop="element"`,
it reports the argument `tag`, excluding the directive prefix, modifiers and value.
These are product diagnostic ranges; the divergence comparator and budgets do not change.

The pinned reference is eslint-plugin-vue 10.9.2 with ESLint 10.4.1 and
vue-eslint-parser 10.4.1. Its filter rule reports `VFilterSequenceExpression.loc`.
Its router rule reports `VAttribute.key` for static attributes and
`VDirectiveKey.argument` for bound attributes. The corresponding authored Relief
locations are `SimpleExpressionNode.loc`, `AttributeNode.name_loc` and the
directive argument's location.

`crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans` retains all 11
original files with byte lengths, SHA-256 hashes and Git blob identities. It also
retains all 28 original divergent diagnostic entries and complete reference packets
for those files plus nine minimal cases, including CRLF, Unicode, multiline
expressions, chains, static and bound attributes, and clean inputs.

`tests/tooling/patina-deprecated-span-oracle.test.mjs` replays the pinned official
provider and compares every packet field. Only the temporary checkout prefix in
`filePath` is recorded as a stable path. `deprecated_diagnostic_spans.rs` checks
the complete native result against those reference ranges, including every native
diagnostic field, help, labels, fixes and counts. It translates the provider's
UTF-16 columns to native byte offsets. Existing expression classification and
dialect tests remain applicable.
