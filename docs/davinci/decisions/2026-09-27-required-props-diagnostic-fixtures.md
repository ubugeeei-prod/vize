# Required-prop boundary diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Historical fix: `f3a26b0e30c98b135a9e897c3584526eb0a2b96d` (#3581).

## Decision and scope

Freeze the complete current production batch diagnostic list for the exact
seven original inputs of `attribute_only_required_props_edges.rs`. The fixture
uses real Vue 3.6.0-beta.10 and TypeScript 7.0.2. Its metadata declares required
T1 execution; the ordinary test body always executes the checker. The exact
runtime test name must be deferred by T0 nextest selection and required by full
T1 Cargo execution.

The original assertion kept 24 file/code/start identities, then checked only
message substrings. The actual capture matches all 24 original identities and
now preserves every full message, severity, file, authored UTF16 start and
returned order. The fixture covers missing required siblings, incorrect named
values, union discriminants, fallthrough attributes, spreads and their order,
dynamic/reserved bindings, component/VNode listeners, callback props, global
custom props and a duplicate named attribute. Successful usages contribute no
diagnostics to the complete list.

The `🎉✅` prefix in Parent.vue line 54 demonstrates the real authored UTF16
column: the Child tag is diagnosed at column 24, while byte and Unicode scalar
columns would be 28 and 23. This is an actual checker diagnostic, not a supplied
mapping record.

Returned order is intentionally preserved. The generated missing-prop checks
for lines 31–33 appear after the line-64 duplicate-attribute diagnostic in the
current result. No sorting is introduced to hide that observable ordering.
Private helper names and all multiline message text remain exact.

An optional test-only capture directory serializes the actual public batch
rows after the unchanged checker completes. Capture writing never replaces
the equality assertion, even while initially collecting a fixture. It records
only fields present in the public result; it is not a raw backend response.

## Verification and remaining work

The initial observation produced 24 real errors and intentionally failed the
empty draft expectation. Its exact identity set was checked against the old
24-record assertion before those actual messages and their original order were
frozen. Fresh repeated runs matched that frozen full batch list, and both
previous fixture packs remained green. Changing the astral-prefix start to
either byte or scalar columns failed. Truncating a multiline message, sorting
the rows, adding a duplicate, or removing a row also failed. The restored repeat
passed. All seven source carriers match their original authored input bytes;
strict Clippy passed.

End ranges, related information and untouched backend payloads remain absent
and explicitly declared. Full #3581 bundled-commit admission, Actions/T1 queue
verification, shared differential execution and native L4 parity remain open.
Native acceptance is zero. This fixture does not constrain future generated
TypeScript text independently of the diagnostic contract. #6879 stays open.

## Exact T0 deferral

Stack this slice on the definite-assignment draft #6939. Extend only the present
package `vize_canon`, binary `fix_history_diagnostics`, exact test
`required_props_keep_exact_unicode_diagnostics`. Prior Canon/Corsa names remain exact;
source existence and T1 metadata are checked. Full Cargo T1 stays unchanged.
Draft Actions and CI parent main/full queue proof remain pending.
