# CSS display suggestions for deep and slotted subjects

Issues: [#7976](https://github.com/ubugeeei-prod/vize/issues/7976) and
[#7984](https://github.com/ubugeeei-prod/vize/issues/7984).

Both reports describe `css/no-display-none` recommending `v-show` for content
selected across a component boundary. Keep both distinct complete originals:
ScheduleView (366 bytes) and DataCalendar (124 bytes), MyField (294 bytes) and
MyInput (140 bytes), and each original 86-byte config. The issue bodies and all
six input hashes are retained under `crates/vize_patina/tests/fixtures/issue-7976`.
GitHub's verified reporter is ubugeeei, public ID 71201308; the reports do not
supply another person's identity or a named source repository.

The existing rule excludes pseudo-elements and conditional CSS blocks. Its
ordinary style branch still recommends an element directive for the parsed
Vue `:deep()` and `:slotted()` subjects in these originals. Extend only that
branch's target ownership check, over the already parsed selector AST and the
existing nested-rule walk. All alternatives of a selector list must select an
external subject before suppressing the recommendation. Propagate that context
through nested descendants and layers. `:is()` and `:where()` require unanimous
external alternatives; do not treat `:has()` or `:not()` filters as the subject.
A sibling relation does not prove the selected sibling belongs to the external
subject, so retain the ordinary recommendation there. Quoted attribute values,
comments and arbitrary custom pseudo names are not ownership markers.

Preserve existing local/root selectors, mixed selector lists, ranges, messages,
severity, help and ordering. `:global()` can select an element in the current
component and does not prove external ownership; keep its diagnostic. The
Issue #7976 global-selector proposal remains unfinished. Existing parser flags
and invalid-CSS fallback are unchanged: this is not expanded support for the
legacy `>>>` or `/deep/` parser grammar. Parsed `::v-deep` / `::v-slotted` retain
their existing pseudo-element handling and pass their external nested context.

Add 47 independently authored complete regression vectors: both original SFCs,
local and nested positives, mixed alternatives, descendant and sibling cases,
relational filters, global/root controls, token lookalikes, legacy functional
forms, layers, conditional/pseudo controls, ordered findings and UTF-8/CRLF.
Rust compares the complete public CSS result at a nonzero offset, the complete
SFC `LintResult` and full JSON report over three unchanged queries. Each CSS
input must actually parse, so an empty oracle cannot pass via parser fallback.
Every rule-off result is also a whole empty `LintResult`.

The ordinary source-CLI observer requires the authenticated compiled build
receipt, then makes 190 calls: two literal original plain commands, three full
JSON queries and one off query per case. Keep each whole stdout/stderr/status,
config, source and expected object before assertions, including failed runs.
Check unchanged source and config bytes and both original neighboring children.
This is source-built legacy correctness evidence, with no new native provider,
serialization, extra parse, numeric cap change or migration claim. No compiled
runtime or Actions pass has been observed during private preparation. Required
source Actions and the full protected queue remain future acceptance gates.

Keep this private until an existing product PR actually merges, as directed by
the maintainer's finite backlog boundary. Root owns publication. Do not close or
deduplicate either report based on this preparation or the anonymous real-app
provenance. Keep #7963 (Nuxt final-comma preservation) and #7938 (generic-attribute
import reads) as unowned future candidates; their current source causes have
been inspected, and their runtime status is unknown.

Private review found two opposite nesting boundaries in the first 6591051 source:
`:is(& + .local)` can select a sibling outside the inherited external subtree,
while nested `.header + .body` stays within that subtree. The locked parcel
selector parser adds an implicit nesting prefix only when no nested `&` occurs;
`:is()`/`:where()`/`:has()`/`:not()` propagate that nesting state. Follow the
actual prefix and unanimous subject alternatives, without a blanket inherited
fallback. A sibling crossing is uncertain until a child/descendant step proves
a shared ancestor. Add 14 complete controls for these boundaries; preserve all
33 prior case objects and both full original reports. This is a source-review
correction, with no compiled acceptance transferred from the old private tree.
