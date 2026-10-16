# Filter expression and router tag report ranges

Issue: [#9218](https://github.com/ubugeeei-prod/vize/issues/9218).

The enforced Real Project Matrix found 14 location-only false-positive / false-negative
pairs in `PanJiaChen/vue-element-admin` at fixture pin
`6858a9ad67483025f6a9432a926beb9327037be3`: 13 filter interpolations and one
`router-link` tag attribute. Bound filter values already used the expression range.
The same range defect also produces two pairs in ZY-Player and 29 in Habitica
(25 filter interpolations and four router tag attributes).

For `{{ message | capitalize }}`, `vue/no-deprecated-filter` now reports
`message | capitalize`, excluding the braces and surrounding whitespace. For
`<router-link tag="button">`, `vue/no-deprecated-router-link-tag-prop` reports
the attribute name `tag`. For `:tag="element"` and `v-bind:tag.prop="element"`,
it reports the argument `tag`, excluding the directive prefix, modifiers and value.
These are product diagnostic ranges; the divergence comparator and budgets do not change.

The pinned reference is eslint-plugin-vue 10.9.2 with ESLint 10.4.1,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0. Its filter rule reports `VFilterSequenceExpression.loc`.
Its router rule reports `VAttribute.key` for static attributes and
`VDirectiveKey.argument` for bound attributes. The corresponding authored Relief
locations are the filter input AST within `SimpleExpressionNode.loc`, `AttributeNode.name_loc` and the
directive argument's location.

`crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans` retains all 11
original vue-element-admin files, two ZY-Player files, 14 Habitica files and one
Frappe Builder file with byte lengths, SHA-256 hashes and Git blob identities. It also
retains all 91 original divergent diagnostic entries and complete reference packets
for those files plus 21 minimal cases, including CRLF, Unicode, multiline
expressions, chains, static and bound attributes, and clean inputs.
The four compressed original Matrix linter reports retain the exact raw producer
bytes and every field for all 655 ordered file packets, including findings from
unrelated rules. Their raw byte lengths and SHA-256 identities are checked too.

`tests/tooling/patina-deprecated-span-oracle.test.mjs` replays the pinned official
provider and compares every packet field. Only the temporary checkout prefix in
`filePath` is recorded as a stable path. `deprecated_diagnostic_spans.rs` checks
the complete native result against those reference ranges, including every native
diagnostic field, help, labels, fixes and counts. It translates the provider's
UTF-16 columns to native byte offsets. Existing expression classification and
dialect tests remain applicable.

Four official packets also contain ESLint's host-level "Unused eslint-disable
directive" warning for `no-alert`. The complete warnings, including their fixes,
remain in the official oracle and its replay assertion. The native adapter
identifies exactly these four administrative warnings separately: they have no
rule ID and are outside the two selected Vue rules. Every selected-rule range,
message and native field remains asserted.

The Frappe Builder finding is a different case. Its
`activeScript.script_type as 'JavaScript' | 'CSS'` binding is a TypeScript
assertion union. The pinned Vue parser splits the union as a filter when using
the TypeScript parser; the same minimal input without that expression parser
produces no filter diagnostic. Both complete official packets are retained.
Native tests explicitly require a completely clean result for the minimized
assertion and the full original Frappe file, preserving #7911's type-grammar
handling. This source-correctness distinction remains visible as a reference
disagreement; this patch does not disguise it through comparator changes.

Grouped filter inputs need one additional range step: `(value) | format` starts
at `value`, while `(value).field | format` starts at the member expression's
opening parenthesis. A cold OXC parse of an already-flagged parenthesized input
provides this inner-expression span. Six minimal cases distinguish grouping,
nested grouping, a grouped binary expression, a conditional, a call and member
access. Habitica's multiline boss-health expressions exercise the same rule on
unchanged original source. The common expression path stays lexical.

The input-prefix parse and the existing assertion-union parse both use L0's
`expression_is_safe_to_parse` guard before entering OXC. Guard refusal retains
one complete lexical filter diagnostic; public-linter controls at 32 and 4,096
levels, with and without an assertion union, exercise that fallback. Five
complete official cases cover named/numeric encoded parentheses in text and
bound attributes, including an encoded leading space. The decoded input AST
start is projected through the L1 entity decoder back to its full authored
spelling; it is never added directly to an authored span. A decode mismatch
retains the lexical range instead of manufacturing an offset.
