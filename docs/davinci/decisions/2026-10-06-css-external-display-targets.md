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

Add 54 independently authored complete regression vectors: both original SFCs,
local and nested positives, mixed alternatives, descendant and sibling cases,
relational filters, global/root controls, token lookalikes, legacy functional
forms, layers, conditional/pseudo controls, ordered findings and UTF-8/CRLF.
Rust compares the complete public CSS result at a nonzero offset, the complete
SFC `LintResult` and full JSON report over three unchanged queries. Each CSS
input must actually parse, so an empty oracle cannot pass via parser fallback.
Every rule-off result is also a whole empty `LintResult`.

The ordinary source-CLI observer requires the authenticated compiled build
receipt, then makes 218 calls: two literal original plain commands, three full
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

Independent review of private 19b6f55 then found that a blocked inner deep
marker must not prevent proof from a farther outer ancestor, for example
`:deep(.outer) .header:deep(.inner) + .body`. Continue through blocked markers;
accept only an unblocked positive ownership proof, otherwise retain the finding.
Seven additional whole controls preserve all 47 prior case objects. This source
correction has no runtime acceptance from either earlier private preparation.

The maintainer's subsequent explicit scaling decision lifts the earlier private
publication boundary. Publish this reviewed slice as the genuine child of
CLI #8086 / Issue #7995 at `4317e5e6fc4b39cf850b2f1c3cf1ab233595b635`,
which itself descends from CI #8073 at `d836c174f1fe5f1c7b8e12219366e137c129048d`.
Preserve all incoming source and decision bytes, the prior ab460 source/corpus
objects, both original bodies/configurations, and the complete 54 API/JSON cases
and 218-call observer. The source-only peer verdict is not compiled acceptance.
The bottom owner registers all three PRs as one native Stack after this child's
PR exists, then verifies the same Stack ID and ordered positions. No individual
auto-merge is permitted. Current Actions must prove this literal source.

The initial parent predates current main and the security remediation #8080.
Retain any authentic advisory failures rather than accepting a stale parent
pass. After #8080 actually merges, the bottom owner replays onto genuine fresh
main once, then each child replays its owned slice onto that actual parent and
reruns Actions. Admit only an exact-green contiguous prefix with `gh stack
merge`, and track protected full suites, original proofs, actual signed merges,
and root-owned public release. No local native build or extra campaign is added.

The initial #8086 native test compilation rejected its new SHA256 helper's
LowerHex formatting before runtime. Its owner repaired only byte-wise hash
encoding and paired qualification docs in successor
`5d0a8adb32aee33a40004676714796b3512e58ec`. Genuinely replay this CSS child
onto that exact parent before initial publication, preserving every incoming
byte, all reviewed source/reference objects, and the original 4317 witness.
Current source/native execution remains pending; no old-parent pass transfers.

Initial CSS #8091 source Check37412533455 is terminal failure. Both its Rust
whole-report law and strict CLI JSON comparison reject the newly authored
ruleDocsPath on the first local control; unchanged output/shared.rs maps the css
namespace to docs/content/rules/musea-and-css.md, and output/json.rs uses that
function directly. Independent primary-source review authenticates those two
complete files at signed143, parent5d0 and CSS74940; css.md does not exist.
Correct only those 27 authored scalar paths. Retain all 54 before vectors,
including the original 33/47 objects, byte-for-byte in
cases.before-docs-path.json with their original hash. A pure delta law permits
only these fields, retaining all sources, spans, order and other whole-report
values; source.json pins both references and the unchanged source authority.
This is a reference authoring correction, not re-recording runtime output or
changing production documentation metadata. Retain the authentic failed raw
job logs and require new exact-head full54 API/218 CLI Actions. The source build
compiled, but no whole54/218 acceptance, protected gate or delivery transfers.
The Stack remains Draft/offqueue until actual security-main incorporation and
fresh whole-chain qualification; both Issues remain open and root publishes.

## Genuine delivered-security replay

Paired decisions for [7976](https://github.com/ubugeeei-prod/vize/issues/7976#issuecomment-6009837099)
and [7984](https://github.com/ubugeeei-prod/vize/issues/7984#issuecomment-6009837400)
record actual signed8080 main48cb and the true CLI8086/d790 parent on bottom
8073/38e. The maintainer delegates existing child delivery after its previous
agent ended. Preserve all five original author/date/message/trailer records,
all54 cases/218 CLI observations, both originals/configs and the before54
archive/sole27 documented path corrections. Every source/oracle/helper/census
body remains exact3cd; incoming objects and canonical350 remain whole.
Retain the authentic old docs-path/security failures and prior scoped runtime
receipts, including the complete218 CLI proof
`e998807ede8193a31e1940f910d6af85fdc4d73c98eac067d1e93ba37915d183`. Fresh automatic whole54/218/source gates,
same ordered Stack8092/protected budgets/signed delivery/root public release
remain required; no old execution transfers or production/workflow changes.
