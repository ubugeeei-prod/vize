# Boolean attribute autofix

This is the smallest independently reviewable slice of
[#7905](https://github.com/ubugeeei-prod/vize/issues/7905), paired with the
[issue decision](https://github.com/ubugeeei-prod/vize/issues/7905#issuecomment-6006082695).
The current `vue/no-boolean-attr-value` rule declares itself fixable but emits
no edit for the original `disabled="disabled"` warning.

Retain the existing native-tag/static-attribute selection and all diagnostic
text, help processing, severity, ranges and ordering. Attach one deletion from
the end of the proven authored attribute name to the end of its assignment.
Validate the source slice and following separator so the fix cannot merge two
attribute names. Preserve every surrounding source byte, including adjacent
attributes, Unicode, CRLF and quote spelling outside the removed value.

[Boolean attribute presence](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attributes)
represents true even for an invalid string such as `"false"`; the edit keeps
that presence. `hidden` is enumerated, and
[`until-found`](https://html.spec.whatwg.org/multipage/interaction.html#the-hidden-attribute)
represents a distinct discovery state. Keep the existing warning but offer no
edit for that state, including case-insensitive and entity spellings. Revising
that historical warning policy is a separate decision.

Seven complete original carriers retain the report body, shell command,
CardList/MyCard/config, original actual output and requested four-rule output.
Their byte hashes and verified public reporter `ubugeeei`/71201308 are recorded
in `crates/vize_patina/tests/fixtures/issue-7905/source.json`. Expected bytes
were authored from this contract, not copied from current runtime output.

Twenty-one whole public-result vectors cover SFC, standalone HTML, JSX and
TSX: original input, quotes, unquoted/empty values, multiple attributes,
UTF-8/CRLF/assignment whitespace, hidden refusals, components, dynamic bindings,
name case and namespace controls. The Rust law compares all public result
fields, including every Fix/TextEdit, applies the full edits, and checks three
stable re-queries. Suppressing help must not suppress an edit. Existing JSX
input/range/message/boundary assertions are conserved; only the two intended
no-edit metadata expectations now assert an edit. This is an intended legacy
fix-metadata change, not native migration credit.

The CLI observer requires the existing exact source-build receipt and retains
all raw streams, statuses, authored input/config hashes, whole JSON and fixed
file bytes before assertions. Its 88 calls include the original four-rule
configuration with all three unrelated findings retained after the boolean
edit, plus each isolated public-entry vector and three stable fix passes.
The original JSON serializer keeps its existing schema without fix metadata;
Rust Fix fields and actual file edits establish the editable behavior.

Reference-integrity and configured JS lint pass locally. Rust/compiled CLI,
exact source Actions, protected full suites and all unchanged instruction
budgets, actual signed merge and installed public replay remain pending.
The finite 0.434 release hold is historical. Deliver this independent slice
from actual current main, with fresh Actions and a protected independent merge;
the root #8074 documentation receipt repair is not a source dependency.
No local native build, installation, performance claim or new stage is added.

The first private reference-integrity run rejected an overstrict uniqueness
assertion on the two original `<MyCard>` openings. The authored empty-first
opening range remains unchanged; the helper now verifies the selected exact
source slice. An explicit lexical comparator also removed a configured lint
warning. Fresh reference checks pass; neither failure supplied runtime credit.

Independent source peer review of the prepared `65439c12c3` confirms the sole
production change and all original/corpus conservation, then identifies two
observer gaps. Pin and compare the entire expected empty stderr from the existing
JSON/help-none CLI contract, and persist the raw process status/streams before
reading the output file so a failed rewrite/read cannot erase process evidence.
Attach and persist full output bytes before assertions. Keep all 21 vectors,
88 calls, complete stdout comparisons, product/provider routes and budgets.
The source replay incorporates actual main `143c1d4a9f`; every incoming canonical
decision stays byte-exact apart from this slice's owned clause. Fresh source
Actions, actual native CLI, protected gates and public replay remain pending;
the private reference checks grant no runtime or release acceptance.

The [current-source observation](https://github.com/ubugeeei-prod/vize/issues/7905#issuecomment-6007681158)
binds reviewed head `437bb51ac8`/tree `96b79a4b1f` and hosted merge `74806e4495`:
the latter has actual `143c1d4a9f` and the reviewed head as direct parents, with
an identical tree. [Check 37399663770](https://github.com/ubugeeei-prod/vize/actions/runs/37399663770)
passes all four Rust shards and the complete 21-case result/edit/fixed-byte law.
Hosted source-built CLI SHA256 `1868430746d8266092563848666a36fbb58707c5ad01e3581b8664a82ac5498a`
executes all 88 calls; the complete 168,199-byte observation artifact SHA256
`7a5eff70ae152168c015a6e98b1c7364dca8bfb58d45d4235b797a5ebcd20822`
independently matches all whole JSON, empty stderr, process state, fixed bytes
and original/config custody. This is that immutable source's runtime credit.
Overall Check is rejected by shared new npm advisories; the final report
faithfully rejects security-audit after successful capture/inventory and
actual worker steps. No waiver or protected admission follows from the passing
runtime. Keep the PR Draft/outside the queue, incorporate the dependency
repair only after actual merge, and require fresh source/protected gates.

Prepare the identical 22-scenario/four-call installed consumer handoff;
change only distribution identity/custody, preserving all whole original
vectors. The adapter is unexecuted, requires the release owner's authentic
registry/tag/source receipt and grants no installed-public/build credit.
Actual merge/publication and the remaining rule gaps stay unfinished.

TODO: the other missing edits for `html-self-closing`,
`component-name-in-template-casing` and `v-slot-style`, the unobserved Vapor
rule, and repeated fix passes (#7906) remain unfinished. Do not close #7905
for this one-rule slice. Include the verified reporter Co-authored-by trailer
on the meaningful source commit and inspect actual final attribution at merge.

The [security-main incorporation](https://github.com/ubugeeei-prod/vize/issues/7905#issuecomment-6009732847) follows actual signed #8080 merge at 2026-10-06T05:04:12Z, main 48cb1d4f35ebd0aa3824817c67752b5e4d0a9dbb, with verified signature and sole incoming143 parent. This existing independent slice is genuinely rebased onto that source. The whole original21 laws/seven carriers/CLI88 and reviewed rule/observer bytes remain unchanged; every incoming decision is conserved. Initial437/748 runtime stays historical, with no combined-source transfer. Require fresh current Actions/security/native21/CLI88 and unchanged protected full/performance/instruction gates, matched squash auto-merge, actual signed merge and next finite publication/installed88. No new approval fence, local native build or waiver; the other three edits/Vapor/#7906 remain unfinished and #7905 stays open.
