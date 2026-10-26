# Native Document implied ends, tables and profile rules (#6835)

The Document-profile structure builder now reads its host rules from
`P: Profile` instead of hard-coding them. `html_structure` instantiates it
with `Document`; unit tests also build the same event tape with `Component`,
so each rule is pinned under both profiles:

- `FOLD_NAME_CASE`: HTML names match without ASCII case under Document and
  exactly under Component.
- `SELF_CLOSING_ANY_ELEMENT`: a slash closes any element under Component;
  under Document it closes only void elements and is ignored elsewhere.
- `IMPLIED_END_TAGS`: under Document, start and end tags imply the ends of
  `p`, `li`, `dt`, `dd`, `rb`, `rt`, `rtc`, `rp`, `option` and `optgroup` as
  the HTML "in body" mode does. Under Component an end tag must name the
  current node, which keeps Vue's authored nesting.
- `TABLE_CONTENT_MODEL`: under Document, `table` enters the HTML table
  insertion modes (table, row group, row, cell, caption and column group).
  Row groups, rows, cells, captions and column groups close each other as
  the browser does. Under Component table parts are ordinary elements.

The admitted set grows by `p`, the `ul`/`ol`/`menu`/`dl` lists and their
items, ruby text, `option`/`optgroup` outside `select`, and every table part.
An element closed by an implied end has no closing span, and its child range
ends at the token that closed it. Text after `</body>` refuses node readback
on the element open there, which may be an implied-end descendant of body.

Only transitions without parse errors are admitted. The browser tree would
otherwise stop following authored tags, so these cases refuse with a typed
reason: `ImpliedStart` (an implied `tbody`, `tr` or `colgroup`),
`FosterParenting` (content inside a table that the browser moves out),
`TableText` (non-whitespace text in a table context), `IgnoredTag` (such as a
table part outside a table) and `ImpliedEnd` (parse-error or stray ends).
`DocumentTreePolicy`, `UnsupportedPolicy` and `unfinished_tree_policies` are
removed because no profile rule is left undeclared. `select` content,
foreign content, raw text, formatting elements and templates still refuse
as unsupported elements.

Twelve complete sources in
`davinci/vize_l1/tests/fixtures/document/html-implied-ends-and-tables.json`
pin whole ancestry, attributes and direct child nodes. Native Rust and real
Chromium `DOMParser` compare against the same packets, and native Rust also
pins which elements close without an authored end tag. Eighteen refusal
controls cover each typed reason.

This does not switch a product route or complete petite-vue semantics.
#6835 and #6843 remain open.
