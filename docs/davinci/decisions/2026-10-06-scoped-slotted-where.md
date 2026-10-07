# Scoped functional and slot selectors

Issue: [#7942](https://github.com/ubugeeei-prod/vize/issues/7942).
Reporter: ubugeeei (GitHub ID 71201308).

A flat style block containing `:slotted()` selects the existing Vite selector
writer. That writer inserted scope attributes before a bare `:where()` and
scoped ordinary compounds preceding slotted content. This changed specificity
and broke the original two-rule component.

Keep this selection and correct the writer. A leading `:where()` or `:is()`
recursively scopes its branches while it remains the scope anchor; a later
ordinary class, type or attribute owns the anchor instead. Slotted targets use
the `-s` identity inside that same functional anchor. Ordinary prefixes before
`:slotted()` remain unscoped. Existing deep/global handling stays on its current
route. This uses the current selector helpers and emits in the current pass.

The primary reference is [Vue 3.5.38 pluginScoped.ts](https://github.com/vuejs/core/blob/v3.5.38/packages/compiler-sfc/src/style/pluginScoped.ts).
Its `rewriteSelector` clears outer injection at `:slotted()` and recursively
rewrites the selected `:where()`/`:is()` node and the slotted argument. A class
before the functional pseudo or after it prevents that pseudo from owning scope.

The original complete issue body and SFC are retained in
`tests/_fixtures/differential/compiler/scoped-slotted-where-7942`. Four original
CSS table inputs and four authored anchor/list/escape controls have complete
selector and declaration references. The old two focused test files are
archived byte-exactly. Their original inputs remain unchanged; only the
incorrect ordinary-prefix attributes in the expected slotted CSS are removed.
This correction follows the primary selector contract, without recording Vize
output as an expectation.

The existing attribute writer was extracted in a separate move-only commit.
The parent oversized source shrinks; the new module and tests stay below 350
lines. The genuine per-crate consumer inventory is regenerated over the full
SFC crate with other shards untouched.

Prepared checks cover the whole pipeline CSS, both minified/non-minified
complete Rust and public NAPI CSS results against authored scoped references,
the original whole SFC through the public Rust and NAPI APIs, complete repeat
results and additive original-source maps, and the real Vite CSS wrapper.
These public native-addon checks exercise its existing SFC compiler route;
they grant no level-native acceptance beyond the actual admitted native scope.

Hosted exact-head source checks, the full existing compiler corpus, protected
instruction gates, actual signed merge and an installed release replay remain
pending. No benchmark/ranking or performance result is claimed. Root owns
publication and the public replay handoff.

The first exact-head source build at `10b9afef` failed before tests: Rust 1.98
Clippy rejects six unchecked string slices in the new functional helper. The
correction uses the existing pseudo-function parts and checked UTF-8 access.
All original sources and complete expected CSS/API vectors remain unchanged;
the failed build is retained and grants no execution acceptance. The separate
inherited dependency audit failure remains with the security lane. Fresh
exact-head source and protected checks are still required.

The same first tooling attempt failed before the new assertions because the
tests package does not declare `@vizejs/native`. Its direct import now uses the
existing checkout-native entry, matching other native tooling tests. The real
Vite wrapper still resolves its declared native dependency. The unchanged
registered task builds that checkout addon first. The complete 872,085-byte
failed tooling log is retained; this repairs test provisioning without changing
any compile option, assertion, input or expectation.

At `ff46327d`, all eight complete CSS/native/Vite comparisons passed, then the
original whole-SFC CSS assertion found one incorrectly authored terminal LF.
The unchanged writer's `flush_declarations` trims final root trivia and drops
empty trailing declarations; the existing slotted style route uses that writer.
The corrected reference removes exactly this one byte and retains every leading,
inter-rule, selector and declaration byte. Its old whole reference/hash and the
872,512-byte failed log are preserved. Original inputs, all eight controls,
production and whole comparisons remain unchanged; no output was re-recorded.
Fresh whole-SFC, map, Rust and protected acceptance remain pending.

The same first complete Rust execution also found two additional old focused
slotted expectations with ordinary-prefix scope attributes: the mixed SFC
assertion and the SFC snapshot. The paired CSS snapshot has the same independently
proven mistake. Remove only those attributes under the same Vue primary contract;
full old files/hashes are retained, with all original inputs and other expected
bytes unchanged. No blanket snapshot update is used.

The L3 remarks sweep additionally requires the new original `App.vue` in its
registered file list. Append only that sorted path: all existing 451 paths and
entire entries/explanations remain byte-exact. The authentic execution reported
452 files, 150 remarks, 15 applied, 135 missed and zero remark changes. This is
fixture membership conservation, with the unchanged full gate still required.

The current native-phase run found the parallel L2 sweep's same new membership
and two new original-file remarks. Independently retain the original div's
authored 13..44 span: its static class props hoist, while its slot child blocks
whole-subtree hoisting under the existing producer. Add only this path and those
two complete literal rows. Every old L2 path/remark/explanation remains byte-exact.
The complete 724,241-byte native failure log is retained, with no parser/pass,
regression explanation or update-mode change. Fresh unchanged gates remain
mandatory; the first native attempt stays failed.

The `769180f` affected build/Clippy and all four Rust shards passed. Its tooling2
run then exposed an incorrect new map premise: the original has no script, and
the unchanged template-only compiler returns no module map. Require both map
fields absent and whole maps-on/off result equality. The earlier prepared
authored-map claim does not apply to this original. Every other field, original
source, CSS reference and repeat comparison remains unchanged. The complete
871,671-byte failed log is retained; no map producer change is included.

The parallel native run already matched all 1,041 L2 remarks with zero changes.
Its next gate found the generated backlog stale. Derive the complete document
from the reviewed baseline using the existing grouping and formatting rules,
first reproducing the entire old document exactly. Only its summary and the
existing slot-blocked subtree reason's eight-to-nine hit/file counts change;
rank, first example and all other bytes remain unchanged. The complete old
document and 400,489-byte failure log are preserved, without compiler/update
mode or captured-output authoring. Fresh whole gates remain mandatory.

The existing #8130 source is refreshed by merging the genuine signed main
`8d33f844fd139f451b15e66ceb7834b2544a6877` into the unchanged original
`d9f2ffa5a1e1781e3a8032ee48b8345551089ee5` lineage. Every original product,
test, fixture, full reference and archived decision is retained. All incoming
main paths are retained, including the OXC vendor, current compiler changes,
native workflow helpers and the patched shell-quote 1.11.0 lock/override.

The only source-union conflict is the generated missed-remarks backlog.
Reproduce both complete prior documents using the existing grouping/order
rules, then derive the exact merged literal baseline: 498 files, 1,087 remarks
(199 applied, 888 missed). All old and incoming membership, remark entries,
explanations and their order remain intact. The added original App.vue still
owns only its two previously reviewed remarks; no expectation or compiler
output is re-recorded. The complete current SFC inventory is derived from its
actual source, with all other shards preserved.

The old exact-head security failure is retained: production npm audit rejected
critical GHSA-pqg4-j6r4-53mv for shell-quote 1.9.0 in all three bounded attempts.
Cargo audit was skipped. The current-main patched dependency is brought in
without widening the reviewed advisory set, changing printed audit packets,
or borrowing older security acceptance. Fresh exact-head Actions, the full
protected compiler/native/instruction suites, actual signed merge and an
installed public replay are still required; root remains the sole queue and
release owner.

The fresh prepublication readback then advances to signed
`523d1523a5285796addeaed2adfed9341bf46d0a` (#8208), whose actual parent is
8d33. Merge this genuine main into the prepared 8d33 integration, retaining
all 47 incoming complete props-style CLI fixtures, source law and decisions.
This disjoint union changes no owned product, original reference or remarks
entry. The same fresh source/protected/actual/installed qualifications remain
required; no older green execution is transferred.

The final genuine prepublication main is
`67e537cbec83b7971767f7d28d570a34f1a5afcb`, after actual #8174 and #8206.
Retain their complete incoming source, fixtures and decision history through
another clean real-parent merge. Owned product/vector bytes and the derived
498-file remarks union remain unchanged; qualify the final head afresh.
