# Preserve formatter document encoding markers

Issue: [#7870](https://github.com/ubugeeei-prod/vize/issues/7870).

The reported 0.432.0 formatter treats a UTF-8 BOM before a Vue template as
top-level SFC content, adding two newlines after it. The strict JSON parser
rejects the same marker before a JSON value. A document encoding marker must
remain immediately before the first authored character.

Keep the uncommon BOM handling in cold, non-inlined helpers. The SFC helper
formats the remainder through the existing formatter and prepends the exact
marker, then computes `FormatResult.changed` against the original document.
The JSON/JSONC helper does the same without changing either parser's syntax
policy. Interior JSON BOMs and malformed values remain errors. Template text,
CSS formatting, script formatting, sorting and ordinary non-BOM paths retain
their existing contracts.

The shared legacy corpus under
`tests/_fixtures/differential/formatter-regressions/utf8-bom-7870/` retains the
original reported Vue, JSON and TypeScript inputs. Fourteen rows cover both
reported defects, JSONC comments, authored scalar markers, explicit CRLF,
Auto CRLF, comment prologues, ordinary documents and two JSON error controls.
Rust exercises complete public-API output, SFC changed flags and three-pass
fixed points. The source-built CLI law uses the existing build receipt and
compares all document bytes after check/write/recheck. Failed execution keeps
all completed attempts and original process streams in the existing
`target/differential/` upload; error cases must leave the file unchanged.

Source Actions, protected full suites, all 104 unchanged instruction budgets,
actual merge and release publication are required before completion. No
native formatter migration or performance improvement is claimed. Broader
YAML/Markdown support, JSON comments policy, whitespace preservation and
template width remain their separate reported issues.

Source Check `37258203933` at `6b3f02b5ab` rejected the cold helper's
unchecked string slice under the unchanged Clippy string-slice rule. Retain
the original failure and replace only that slice with checked `strip_prefix`;
all corpus bytes, complete comparisons and budgets remain unchanged. Fresh
exact-head source and protected execution are required.

The actual-main replay on `a2712e78968e9112c89cbc2111b108bd0e51959c` preserves every incoming decision, every original author/trailer and all owned production, corpus, runtime/helper and strict witness bytes. The earlier source failure/success receipts remain retained; fresh exact-head Actions and protected delivery are required before requeue.
