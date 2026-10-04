# Original For value alias body DOM proposal

Status: authorized private implementation on literal main `8ae25a1`.
Production and acceptance harness are authored; Rust/source-built capture and
protected acceptance have not run. Publication and new queue admission remain
held by the release cutoff. The accepted static-body slice and its failed-prefix/terminal
receipts stay in the [primitive For record](./2026-10-04-original-for-primitive-dom-proposal.md).

## Concrete bounded input

`<script setup>let count=2</script><template><i v-for='item in count'>{{item}}</i></template>`
needs the original callback value declaration and original interpolation read,
not a script declaration fabricated for `item`. Initial children are `1`, `2`;
mutating the actual setup getter/setter to four and forcing a render produces
`1`, `2`, `3`, `4`. Pinned Vue 3.5.35 uses UNKEYED_FRAGMENT256,
`openBlock(true)`, an element block and `toDisplayString(item)` with TEXT1.
Primitive initializer metadata gives no numeric proof; original `renderList`
continues to determine runtime values.

A primary-only private probe captures six complete JS/TS source/script/render/
map/module records and real initialization/update/unmount for numeric, Unicode,
shadowing, string, null and zero collections. It executes zero native modules
and grants no target admission. Its output SHA256 is
`d29bc8fd750747e06178ce5e8f274aa65839c6f5282c5b44b17fe71a75a4db9e`;
primary helper order is renderList, Fragment, openBlock, createElementBlock,
toDisplayString. Real native code/maps/runtime remain a later acceptance gate.

## Authentic provider and consumer boundary

All required owning providers already exist on literal main: selected normal
setup Program/envelope, attached original For head and Params, distinct alias
BindingId/Scope/TemplateDeclaration, and nested original interpolation with its
once-resolved immutable File occurrence. No extra parse, AST/header/body walk,
name lookup, pipeline stage or neutral expression is needed.

The current first refusal is `dom/build/file.rs`: it requires
`BindingRef.declaration()` (script-only), so genuine template aliases correctly
return None before SelectedReads classification. FileResolution already admits
only the same File and can retrieve authentic alias bindings. A narrow private
branch may admit only the value alias of the currently accepted root For frame:
same completed File, exact attached OriginalFor allocation/head, same
TemplateDeclaration origin/role/Scope, full original parameter and occurrence
name, and the interpolation's real current scope. It must never mint ScriptUnit
provenance or call `NativeSelectedSetup.binding` on an alias as though it were a
setup row. Collection access remains the original enclosing setup Read.

During the existing occurrence loop, store a sealed ForValue classification in
the existing VueRenderRead row; no new field/table/vector/frame or canonical Op
layout is needed. The separate L4 accessor emits the original bare parameter
name only for that classification, with original body reference maps. Existing
setup reads remain `$setup.name` outside callbacks. A generic/ordinary/neutral
analysis cannot select this classification.

The existing Enter may tentatively observe one original interpolation in the
attribute-free direct body Element. After its real read row is classified, only
a normalized Identifier for the current value alias may remain accepted. The
existing Leave validates one dynamic DomText node through its existing row,
without scanning the body again. Preserve current empty/static text bodies.
Do not accidentally admit literal interpolation, outer setup body reads, mixed
text-plus-alias groups, compound/member/call roots, keys, nested/mixed roots,
attributes, events, components or slots. Those inputs keep typed refusals.

The callback must validate every helper spelling actually used inside it:
openBlock, createElementBlock and toDisplayString. renderList and Fragment
remain outside. All existing `_`/`$` aliases keep earliest L2 ReservedAlias,
and strict L1 eval/arguments/yield refusals remain; no wider alias namespace is
proposed and unreachable target collision claims are not made.

## Proposed owned source and qualification

Narrow production changes: `vize_l3/src/decision/dom/build/file.rs`, a new
`build/for_head/value.rs` sibling, `build/for_head/runtime.rs`, the existing
`dom/vue.rs` enum and a small private accessor branch in
`vize_l4/src/targets/dom/vue.rs`/`targets/dom/for_head.rs`. Common setup owner,
L2 factory/resolver/header, shared canonical decision/build.rs and original
attribute-value provider remain untouched. Coordinate any shared DOM Enter
registration with the original-value owner and preserve its same-slot cursor.

Add source laws for same-name shadowing, original alias/parameter/current scope,
foreign equal bytes/IDs, neutral and generic contrast, incomplete/hole/movement/
drop/unwind, and all intentionally excluded body families. Add six independent
whole JS/TS native module/object/raw-map fixtures, pinned complete primary
oracles and actual fresh Vue mount/update/unmount with current-source hash joins.
Mandatory source-built capture must select emitter-only changes and fail on
missing/partial packets. Preserve accepted ten For, seven setup and five click
payloads exactly; all 104 immutable protected caps/ratchets and full suites apply.
No local builds/install or native credit from the primary-only probe.

After implementation authorization and independent review, this can be one small
independent source PR because all actual providers are merged. Register a native
Stack only if a concrete missing provider is discovered. Carry the private
terminal two-document packet with the genuine source change, then follow exact
Actions, protected candidate and actual signed merge. Const/STABLE64, other
targets, broad control/runtime/default and fix-history migration stay unfinished.

## Private implementation freeze obligations

The existing File occurrence loop now classifies only the current attached For
value alias, retaining the exact immutable occurrence, original expression,
TemplateDeclaration/Params, BindingRef and child Scope identity. `ForValue`
uses the existing VueRenderRead row and bare original-name Writer path. Enter
permits tentative interpolation only within the direct carrier; Leave admits
one dynamic singleton whose genuine read has that classification. Static and
empty bodies keep their old path. Every actual callback helper including
`toDisplayString` is checked; `_toDisplayString` remains an earlier L2
ReservedAlias and has a separate original primary/refusal packet.

New owning source laws cover shadowing, foreign equal IDs, copied occurrences,
neutral expression coordinates, a missing genuine read table, movement/drop,
caught open callback unwind, generic/neutral contrasts and excluded body shapes.
The new six-source fixture is separate from the unchanged ten static For,
seven setup and five click packs. It contains complete independent native desired
modules, map objects and serialized raw strings plus full pinned primary script/
render maps. The mandatory Actions capture compares every byte/string/object
before actual fresh Vue setup/mount/update/unmount and current-source hash joins.
Local Node controls execute desired modules and primary references only; they
supply no Rust output/ownership or protected performance acceptance.

No local Rust build/install, provider graft or publication/queue admission is
performed. Independent frozen-source review, exact-head full native captures,
all 104 immutable rows/ratchets and eventual signed merge remain required before
this private slice can claim delivery. Broad loops, const/STABLE64, other targets,
default switches and product fix-history closure remain unfinished.

Pure controls currently pass 38 Node laws across the new six-source desired
module/primary pack and existing packs/routing. The current private runtime
helper reexecutes accepted `2df4fd2` Rust code for all ten/seven/five fixtures;
all three resulting runtime packet files are byte-identical to that accepted
capture, and all three fixture files remain byte-identical. This conservation
receipt proves compatibility of the harness change only, not a new native
compiler execution. Original callback comments/grouping and semantic source
spellings beyond the normalized Identifier stay refused. Production/new Rust
laws and privacy examples remain authored, not compiled or executed locally.

Paired private decision: [#6839 comment5976393669](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5976393669);
authored production/harness commit `4296329b3f` retains all remaining Actions
and protected acceptance obligations. No PR or queue entry has been created.
