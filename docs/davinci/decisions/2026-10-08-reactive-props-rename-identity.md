# Reactive props destructure rename identity

Owning report: #7996; recursive component shorthand remains a separate #7994
qualification. Start from signed actual main `486c390d81643c16b865754239a987cfe4c9861e`;
the following signed `eed471b4` adds no rename/reference/fixture changes.

A public prop and the binding created by reactive destructuring have separate
rename roles. Public rename from either the Child type key or Parent attribute
must update the type key, destructure property key and matching attributes,
preserving the Child local alias and Parent local value. Local rename from
either the binding declaration or template value must update only that local
identity; same-name destructures expand to an alias, preserving the public key.

The complete original report and reported Child/Parent source are frozen under
`tests/_fixtures/differential/lsp/reactive-props-rename/7996`. The new actual CLI
stdio target `lsp_reactive_props_rename_cli` retains 24 separate sessions: four
query directions, shorthand/default/explicit-alias bindings and LF/CRLF. It
compares entire references and WorkspaceEdits, applies every returned edit to
complete files, then requires both resulting diagnostic arrays to be empty.
Existing original same-name, mixed-role and alias/external-edit controls remain
unchanged. This initial source records qualification intent; actual current-head
Actions must determine the remaining route defect and qualify any correction.

No old binary, predecessor green run, static concern or partially exercised
session supplies current-source acceptance. Native Stack/queue delivery, actual
signed merge and release remain separate responsibilities owned by the root.
The issue stays open until all reported directions receive genuine qualification.

## Current source reproduction and correction

PR #8188 source `570de01bd80e297e3937ec05ac44619c2d6f0273` genuinely failed
Check 37642663121. Public declaration rename omitted the destructure key and
produced TS2339; local declaration rename additionally changed the public type
and Parent implicit value, producing TS2339, missing-required-prop and unknown
Parent value diagnostics. Failed Rust jobs 112869450696 and 112869450967 retain
complete references, WorkspaceEdits, applied/disk files and diagnostic arrays.
Only the first shorthand/LF session executed in each new wrapper before its
assertion; this supplies no acceptance for the remaining 22 sessions.

The correction keeps all exact reverse-mapped generated identities of a native
resolved authored declaration, so both exported component type and original
setup type reach the native property rename. Complete authored range roundtrips
and existing scope/mapping/batch gates still control every endpoint and edit.
Actual setup BindingIdentifier definitions suppress public endpoint discovery
for local queries; producer-owned component arguments retain the public role.
No posthoc name filter or source spelling search removes a wrong edit.

TypeScript native references at a shorthand binding declaration include its
source-property group even though rename there selects the local variable.
For reactive object destructure declarations only, the reference query therefore
uses a real generated IdentifierReference resolved to that exact local symbol.
Each materialized copy receives the same role selection; ordinary object and
import shorthand retain their prior route. Generated parser/value-resolution
failure after a proved reactive cursor returns an authoritative empty reference
answer. TS/TSX syntax and actual Canon projection controls accompany the native
24-session contract. Setup-role detection alone is not a malformed-parser
fail-closed claim; existing canonical failure/scope guards remain necessary.

Fresh exact-head source/native Actions must qualify the correction and every
unchanged authored vector before readiness, queue admission, merge or closure.

The derived LSP consumer inventory is regenerated with its unchanged generator,
adding only the ten source-import rows introduced by these helpers. The binding
role unit now compares the complete ordered authored range vector; the existing
assertion allowlist and all-target lint requirements remain unchanged. Source
head `0e8c56e874` stopped before the 24-session workers on those source hygiene
gates and a needless borrow in a new unit. It supplies no 24-session acceptance
credit; the corrected head must qualify afresh.

Source head `97a8c2a3ad` passed the public declaration shorthand LF and CRLF
transactions before stopping at the default case's initial diagnostics. Its
input constructor had also replaced the inner braces of `{{ label }}`, making
that derived template an assignment and causing TS2588. This was a test input
defect, not proof of a product default diagnostic. Default and alias cases now
load complete authored supplemental files directly. The original report files
and every complete reference/edit/applied-text/post-diagnostic oracle remain
unchanged; the full 24-session qualification still requires the next exact head.

## Inventory composition and recursive intersection

The complete source head `5caccfb36a` passed its source and configured-native workflows. Its queue admission remained UNMERGEABLE before candidate creation: the authenticated prior prefix `b7f58611236d005628cbe6dfd0c78e1845f7978a` and this head insert adjacent genuine imports into the generated `lsp/vize_maestro.tsv` inventory. A read-only merge-tree confirmed that this inventory was the only conflict. No protected runtime failure or completion is inferred from that admission. The maintainer removed only #8188 while unaffected candidates continued.

After #8190 actually merged as signed `656649c7cfbf5996d9ec8a3d7e8711a14018396b`, merge genuine signed main `b3d837f067147ad3bd01925034983009f918297b` into this existing branch, preserving its qualified ancestry and every incoming change. Resolve the sole conflict with the unchanged complete inventory generator: all 19 artifacts check, and only the ten genuine imports in the Maestro shard differ from main. The public/local identity correction, recursive native role capture, edit-scope guards, and complete original decision paragraphs remain present.

The two previously qualified suites cover reactive destructure and recursive shorthand separately. Add eight narrow configured-native stdio sessions for their intersection: self-recursive shorthand with reactive destructure, public declaration/parent argument and local declaration/template origins, each LF/CRLF. Before any query, author the complete expected reference and edit vectors and full repaired child/parent texts. Public rename retains the local binding, expanding the destructure to `{ heading: label }` and both public arguments to `:heading="label"`; local rename retains both public keys and the parent, expanding the local binding and recursive value to `heading`. Apply the actual transaction to memory and disk, then require both versioned post-edit diagnostics empty. The original 24 and peer 40 full transaction oracles remain unchanged.

This intersection is unqualified until the fresh source producer actually executes it; there is no current product-failure claim and no speculative production change. Any successor requires fresh source Actions and a newly composed protected candidate. Prior source success and superseded candidate receipts do not transfer. TODO: attend full normal native/runtime/performance checks, actual signed merge, fresh-main ancestry, and release.

## Composed native regression and scoped declaration projections

Source `67bb382ef7` passed its configured source workflows and all 32 reactive/intersection transactions, but protected candidate `0d4a89679d58162d477f53103dbce63bb6b2af0b` genuinely failed three unchanged REQUIRE_TSGO controls in Check 37666317611. Model and package rename returned no transaction; a static event rename additionally edited the unrelated CallChild. The full worker receipts, complete JUnit failure text and immutable worker log retain the original assertions. The immediate signed parent `f3ed2ee49cd50619830b3db6a2ae2d946f18ec5e` executed all three same bodies successfully with the native runtime required. The maintainer removed only the failed layer; its 32 transactions and the peer 40/three geometry laws passed, but the full candidate did not qualify.

The new duplicate-declaration projection expansion used a declaration name without a positive component-prop endpoint, extending ordinary exports and model/event symbols onto additional generated identities. Restore the existing definition-verified component-prop endpoint guard. Permit the new exact duplicate-property projections only for ordinary public-prop queries; preserve the old materialized semantic projections for model/event handling. No edit-name removal, assertion/count change, native skip, selector change or gate waiver corrects this failure.

Genuinely incorporate signed actual main `f3ed2ee49cd50619830b3db6a2ae2d946f18ec5e`, retaining the complete prior source ancestry and all incoming changes. Its pre-correction merge tree exactly equals the failed candidate's compiled tree `08cdc8a3fdff3f2eea7be79942285cde5484f7da`. Add the [complete original report controls](./2026-10-08-lsp-original-rename-reports.md) in this same existing #8188 correction: 16 new full stdio sessions with independent version-3 golden repair, preserving the original 32 and peer 40 cases. Original historical configurations remain frozen; the standard current native fixture and its actual configurations are explicitly recorded.

Before readmission, fresh current-source hosted authority must genuinely execute the three old native bodies, all 16 original-report transactions, the existing 32/40 sessions and three geometry laws with REQUIRE_TSGO enabled and DISABLE_TSGO absent. Configured ordinary source checks cannot supply credit for optional native bodies they disable. Both #7994 and #7996 stay open until their complete original contracts receive current native qualification and actual delivery. Fresh protected runtime/corpus/104 instruction ceilings, signed merge, main ancestry and release remain separate requirements.

## Per-unit mandatory native resolution

Source `243f749cf8` passed ordinary source checks, all 16 original complete
transactions and existing 32/40 sessions, native phases and its full Rust job.
However, the three existing model/event/package resolver helpers could return
`None` even when that full job set `VIZE_TEST_REQUIRE_TSGO=1`. Disable overrides,
missing workspace ancestry and resolver errors all permitted a silent early
return from the test body. A separate CLI process's successful native lookup
cannot qualify those individual body entries. Preserve those historical result
observations without claiming their old-three native admission authority.

Genuinely merge signed actual main
`3e0745b6277c00176178de6e9c15de2ee500a258`, preserving previous source ancestry
and every incoming change. Make the three helpers share their existing lookup
through a test-only requirement guard. Under the exact existing
`VIZE_TEST_REQUIRE_TSGO=1` flag, contradictory `VIZE_TEST_DISABLE_TSGO`, missing
workspace ancestry and executable lookup errors fail the individual test before
its early return. After successful lookup, the existing startup, timeout and
query assertions already fail closed. Ordinary optional/disabled native lookup
retains its prior behavior.

Four isolated refusal/compatibility laws exercise an actual nonexistent
explicit executable, absent workspace, contradictory flags and ordinary
optional absence. They do not mutate the process environment or alter any
rename fixture, golden, edit count or native body. The original 16/32/40/three
laws and old model/package/event vectors remain complete and unchanged. Fresh
source Actions and one full Rust execution with REQUIRE_TSGO enabled and
DISABLE_TSGO absent must qualify the repaired source before normal protected
intake; unrelated optional coverage does not add a waiting gate. Protected
runtime/corpus/instruction budgets, actual signed merge and release remain
separate requirements.

## Actual original-report source delivery

PR [#8188](https://github.com/ubugeeei-prod/vize/pull/8188) actually merged at
2026-10-07 22:12:28 UTC as signed
`177933f33e75af1f696fdc468665e7850025d28b`, independently verified on fresh
main. Its [exact protected Check](https://github.com/ubugeeei-prod/vize/actions/runs/37692670456)
passed with current original 16 complete transactions, inherited 32/40 sessions,
three geometry laws, four native requirement laws, per-unit required
model/event/package execution and 104 budgets measured three times. Historical
source and replaced candidate results supply no execution credit.

The original Expected text of #7994/#7996 requires the delivered shorthand
expansions and inline reactive public/local identity with code that type-checks.
The source bugs closed after genuine native execution and actual signed merge;
[paired #7994](https://github.com/ubugeeei-prod/vize/issues/7994#issuecomment-6047945164)
and [#7996](https://github.com/ubugeeei-prod/vize/issues/7996#issuecomment-6047945636)
retain the concrete delivery facts. Installed v0.435.1 replay and publication
remain independent release authority. The configured feature differential job
uses its declared smoke corpus with `closure_evidence=false`; it provides no
full-corpus closure authority. Local named type-alias/interface combinations
are a separate [48-session test-only qualification](./2026-10-08-reactive-named-type-rename-controls.md),
not an invented original source-bug closure condition.
