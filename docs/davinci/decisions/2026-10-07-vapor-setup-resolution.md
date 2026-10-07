# Vapor filename recursion and setup directives (#7884)

Vapor asset resolution already receives the SFC filename-derived component
name and script-setup binding metadata. Use those owned facts: a matching
component tag (including its kebab-case spelling) marks the existing runtime
resolution call as a possible self reference. Imported setup components still
win first. Only experimental `Self` changes the authored resolution name;
ordinary filename recursion keeps the original tag and its source-map anchor.

Custom directives first resolve their `v` + PascalCase setup binding from the
existing setup metadata. Other directives retain normal registry resolution.
Allocate the `resolveDirective` helper only for an actual registry lookup;
builtin directives keep their existing paths. This adds no parse, stage,
serialization or legacy-backed level shortcut.

The complete original recursive Tree, a same-file-name imported-component
precedence control, its Child and mixed local/global directives are retained in
`tests/_fixtures/differential/compiler/vapor-setup-resolution/`. Existing
Actions Rust workers execute development/production × requested inline/separate
whole SFC modules, require equal complete map-on/off result fields, and mount
both source-built output and the locked official compiler/runtime against
independent whole DOM expectations. Recursive root prop updates must preserve
surviving node identity, detach the removed branch and reintroduce it correctly;
diagnostics and unmount are part of every row. Raw inputs/process outcomes are
retained before assertion. The pinned Vue runtime is currently 3.6.0-rc.9 while
the report uses rc.10; the latter remains a separate runtime qualification.

Custom-directive native admission and whole native SFC migration, SSR/hydration,
browsers and performance are not completion claims from this compatibility
slice. Exact-source Actions, protected instruction ceilings, actual merge and
installed publication remain pending. Global directives and experimental Self
controls retain their existing resolution contracts.

Initial `7ae93e5008` Check `37591830338` worker3 (`112697172268`) failed the
new module-map-presence assertion before runtime execution. The authored Child
has an empty setup block: the existing whole-module provenance only anchors
authored script bytes, so Child requires no map while every nonempty original
and control still requires one. Full map-on/off return-field equality remains;
complete compiled packets are now retained before map assertions as well as
before runtime assertions. The locked official parse uses `ignoreEmpty: false`
to preserve that exact empty Child script for compileScript. No reported source
or independent DOM/identity/cleanup expectation changes.

Filename case normalization beyond the reported `Tree.vue` and matching kebab
tag remains unqualified: the SFC owner currently supplies the raw filename stem,
so lower-case/kebab filenames versus Pascal tags need a separate acceptance
control. This TODO grants no closure or native migration credit.

The existing PR now composes actual signed main
`8c7727613de6213e0a52cde96eeb295d01c9bef5` from its real
`3ed19a53f0b80a32583a15ece1196cc985a591e0` source head. Every owned product,
test, original fixture and runtime observer byte remains unchanged; the shared
SFC consumption inventory retains both sides' entries. The coordinated decision
record preserves the complete slice clause and incoming decisions. Previous
source success grants no acceptance to this composition: fresh exact-head
Actions, protected gates and actual merge remain required, with both filename
case normalization and native custom-directive admission still unfinished.

After the protected preceding cohort actually merged, the same local source
chain incorporates signed main `3ed1cc90908c88016310b90a855500c94a156f1f`.
That incoming main changes no owned Vapor or SFC setup source or fixture;
the complete owned byte census remains exact. The approved common 350-line
record preserves all prior map-failure evidence and TODO boundaries. This
genuine main union requires fresh exact-head Actions and protected actual
delivery; projected merge analysis supplies neither ancestry nor execution.

The unchanged original module/runtime controls and source Actions passed on
`5829fd5741dacbfe6dbc1f6b40c8cf95fb13dc93`, but protected candidate
`ebc92600629b6587543048959e3d37fed3bec3aa` exceeded the existing Vapor
lowering ceiling (77,212 > 76,975) and interpolation ceiling
(5,364,859 > 5,340,048), alongside three Croquis ceilings. The candidate was
removed from the queue; these measured failures remain historical evidence.
The corrective source composes genuine signed main
`eed471b4424b922b878bc35cac765dc7274e5736` without compiler product conflicts
and retains the approved complete 350-line record and every original fixture.
The retained following whole-prefix candidate
`1a9b7f1654462485548990e8407bbf6ad0427fa0` contains 500 empty-scope lookups
and no component-resolution call in its interpolation capture. This motivates
shortening the existing scope metadata path while preserving component output.
Empty loop and slot stacks with absent binding metadata now return the same
`None` through a small inline wrapper; every other scope uses the unchanged
resolution body. This adds no scan or allocation. The historical counts do not
qualify this source. Fresh Actions and protected measurements must
establish the resulting counts before delivery; the filename and native
custom-directive boundaries above remain unfinished.
