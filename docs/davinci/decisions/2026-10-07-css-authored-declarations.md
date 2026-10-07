# Preserve authored values while normalizing CSS punctuation (#7866)

Issue: [#7866](https://github.com/ubugeeei-prod/vize/issues/7866).
This recovers the unpushed 2026-10-05 source work into a fresh worktree, retaining
the original worktree and original 27-case authored corpus. It branches from
actual #8178 to reuse the shared source observer's explicit corpus plans; the
native Stack must be registered and merged by the delivery owner.

The existing authored-value fallback leaves every raw `prop:value` and omitted
final declaration semicolon intact whenever a separate value, selector or
header differs from the CSS printer. Recognize the actual property-name colon
inside that existing writer, insert the separator and optional final semicolon,
and preserve the complete authored value. Headers/selectors and existing
comment/SCSS policies retain their ownership. Balanced custom-property brace
values stay opaque, including escaped leading dashes. Empty and whitespace-only
custom values preserve their distinct original bytes. The current tabs/width
continuation writer is retained. No CSS parse, pipeline stage or serialization
is added; the existing printer/token comparison still selects the fallback.

The additive 27-case corpus independently authors exact Plain/Trigger outputs,
all reported trigger classes, implicit nesting, quotes/escapes/Unicode/custom
values, comments, tabs/width/CRLF and compact historical CSS. Actual Rust checks
original/reference CSS parse/print semantics and three complete API passes.
The existing source-built observer reuses its build for 27 API cases and five
complete default CLI check/dry/write/recheck controls. Parent #8178's 27-case
corpus and every incoming source change stay in place.

The original 300 history inputs/captures/outputs stay frozen. Exactly one
compact standalone style coordinate needs an explicit current reference:
`capture/style-block-keeps-box-values-and-implicit-nested-selectors/style/1`.
The current output inserts five property colon spaces and two optional final
semicolons while retaining all five values and the implicit `.item` selector.
Original input SHA256: `7fdf75699b276bde18c066778f12b0d15e375d7918eda21ef4bafc0379cc4643`.
Historical output SHA256: `7834ed0dac79ddab96d60e1bd512d51caf32f699f38cb12260cd38c230f66cc5`.
Authored current output SHA256: `2b44ad9ec1b3f36febe5205b3a381f02fdeebe30707b74c207d8fba4a878182d`.
Authority SHA256: `62f13a706244764aa24ebf1edb1b2f46e087f1dbc5f305c19951ab57a8a30c22`.
The authority pins the original CSS/API/default-options/source witness and all
complete bytes. Current admission must observe the historical mismatch as
`different`, never as a historical match, and separately qualify the complete
current output over all three passes. Existing #7826 SFC and #7877 root-comment
refinements remain separately pinned. The affected pack has 22 strict history
matches, two separately qualified current matches and one internal observation.
Nine local pure history/forgery laws passed; this grants no formatter runtime
credit. Original source-law bodies remain frozen while only the complete
current style owner hash advances for its new private declaration module.

Hosted Rust/API/CLI, protected full history qualification, unchanged instruction
ceilings, actual merge and installed supported-release proof remain pending.
Known unrelated SDK audit failures belong to root's security lane; no allowance
or duplicate dependency change is introduced. URL/comment scanning remains a
separate existing defect. Native formatter migration and fix-history completion
remain unfinished. Reporter credit:
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.

The actual child replay also incorporates parent #8178’s behavior-preserving
value-helper move and strict12 current continuation/control authorities. Both
current style owner pins are recomputed for the composed private module list;
every original source law/input/capture/output and both decision clauses are
retained. The canonical record stays at its unchanged350-line ceiling.

## Current named snapshot custody

Exact-head hosted Rust `47258160b0` exposed one intended live named snapshot
transition in `test_format_nested_css_at_rule`: the recognized `color:red`
declaration now prints `color: red;`. Preserve the complete original `.snap`
under `snapshot-witnesses/nested-css-at-rule.pre-7866.snap` before changing the
live file. Original asset SHA256 is
`aba736ea0f41a939bb08cc84885c82f46b0d80376b6cae388de83107691869d4`;
current complete snapshot SHA256 is
`5ea562b78c25145d1abfccafdf76698168fdc55563a24ccd196c4a633aede4f0`.
The additive `current-snapshot-7866.json` authority hashes both whole assets and
pins L090, its original style owner and the exact unchanged law body. Validation
allows only that one colon space/final semicolon, retains metadata and all rule/
value bytes, and rejects restored old current bytes or archive edits. Original
L090 metadata and all original300 inputs/output/capture/source assets stay
immutable; named snapshot framing gains no public-output byte credit. The
source-witness and current-reference paths enforce this custody. Pure forgery
laws pass; fresh hosted snapshot/runtime qualification remains required. The
child is actually rebased onto parent `454853f3c6`, inheriting the same exact
12-row Rust continuation/rule current-reference controls.
