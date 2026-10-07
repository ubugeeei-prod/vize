# Content Mapper script recovery during unfinished template tags

Owning issue: [#8158](https://github.com/ubugeeei-prod/vize/issues/8158), a bounded
acceptance slice under #3984 and #3953.

The Content Mapper currently sets `preserve_script_on_template_error` to false.
When the parser reports an unfinished tag, the existing virtual-TS generator
therefore replaces the entire file with a fallback module, hiding an unrelated
authored script diagnostic. The editor projection already uses the same
parser-owned recovery to preserve scripts.

Use that existing recovery for the Content Mapper. Its admission requires
`EofInTag` and permits only the established recovery codes or `MissingEndTag`.
The unusable template AST remains absent. Template parser diagnostics remain
visible, and scripts keep their authored code and source mappings. No parser,
pipeline stage, spelling-based semantic engine or alias-rename support is added.

The committed differential corpus covers setup, normal and split scripts. Each
puts an astral character before the diagnosed identifier and runs with LF and
CRLF. The pinned standard tsgo oracle exercises complete, unfinished-tag,
template-repaired and script-repaired states. Every returned diagnostic is
compared; the independent script expectation stays TS2322 at its authored UTF-16
position until the script is repaired. Whole script-only mapper code, mappings
and semantic links are compared through the public transform API.

Existing batch fallback, scriptless-template and complete-malformed-template
controls remain mandatory. The original TS40 malformed template remains on its
existing fallback. #4075's unsupported alias rename remains unchanged.

## Validation

The initial regression-only commit intentionally precedes the production change.
Hosted Actions must demonstrate its failure, then qualify the corrected exact
head. The existing Content Mapper Conformance workflow supplies the pinned
standard-tsgo runtime. Source-only formatting does not establish native runtime,
whole-product migration, 10x performance, release or roadmap completion.
