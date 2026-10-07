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
