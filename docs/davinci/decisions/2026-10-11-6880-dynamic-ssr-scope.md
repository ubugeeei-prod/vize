# Dynamic component SSR scope ownership

[Compiler fix history #6880](https://github.com/ubugeeei-prod/vize/issues/6880)
owns this correction. The original VueFes heading renders a dynamic native
`h1`, `h2` or `h3`, with optional anchor and text or span slots. Cold native
SSR on source `e23560400b7642e4925c98e6da513b628ce64709` attached the
heading's own scope attribute twice in all eight original cases. The
[source run](https://github.com/ubugeeei-prod/vize/actions/runs/38117641371)
retains the complete input, compiled graphs and raw observations.

`createVNode` already captures the current rendering component's scope ID.
The renderer appends that ID when the dynamic component resolves to a native
element. Both SSR emitters also injected that same ID as an explicit prop.
The legacy producer now reserves explicit scope props for static component
calls. The native L4 producer omits this prop only in its dynamic VNode call.
Static component calls, fallthrough attributes, slots and pipeline stages
retain their existing behavior.

## Complete original reproduction

The [whole source fixture](../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/VFHeading.vue.txt)
is the unchanged 1,359-byte original SFC. Its scope companion retains both
the [whole official SSR object](../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/scope.expected.ssr.json)
and the [whole failing native SSR object](../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/scope.original-failure.ssr.json).
[Scope authority](../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/scope.authority.json)
binds their hashes to the original hosted packets and pinned oracle.
Nothing in these observations removes, reorders or normalizes attributes,
markers, text, slots or warnings.

Replaying both original complete compiled graphs with the pinned official
Vue/compiler/renderer 3.6.0-alpha.2 reproduced their exact raw SSR objects.
Removing only the two extra generated scope-prop merges changed the complete
native HTML from 1,501 to the official 1,373 bytes, with zero warnings. This
was a causal graph probe, not a fresh source compilation claim.

The new SFC regression executes the actual compiler on the whole original
input, then renders all four original prop sets with both text and span
slots. It retains the existing public Vite assembly contract: the raw SFC
render module receives `__scopeId` when the scoped component is assembled.
Omitting that assembly initially hid the defect; that preliminary observation
grants no passing source credit. With the actual contract retained, the
unchanged producer fails on the whole original duplicate output, and the
corrected producer passes against the entire original official SSR object.
This local source test uses the declared stable Vue/server-renderer 3.5.35
pair; original hosted alpha.2 evidence and local stable evidence stay distinct.

Native L4 controls require selected emission for the complete heading and
three dynamic neighbors, and compare their entire modules, source maps and
diagnostics with the legacy emitter. Two complete static-component module
snapshots were frozen before either production edit; both remain byte-exact.
The existing 80 SSR library tests are retained alongside these two controls.

## Delivery boundary

This narrow producer change is the third dependent slice after the Vite
stylesheet and Nuxt transport fixes. The composed source must pass the
unchanged complete 38-case VueFes qualifier and normal protected checks
before merge. Historical source runs grant no qualification to the composed
head. Merge, publication and released consumer acceptance remain separate.

This fixes duplicate dynamic scope attributes. It makes no claim that all
SSR output or CSS cascade behavior is complete. The five retained UI markup
failures (media player time, scrubber preview time, signature pad canvas and
the two webcam controls) remain unresolved under their original oracles;
their inputs, expectations and failure ledgers are unchanged.

## Composed source evidence and source gates

The immutable [composed run on `399e71b`](https://github.com/ubugeeei-prod/vize/actions/runs/38120403216)
retains the complete original 38-case workload and its failure artifacts.
The dev home case failed all three original attempts with a diff ratio of
`0.1235105465271594` against the unchanged `0.002` budget; 37 cases did not
run. The stylesheet fixes have not repaired that actual app result.

The same run freshly built the native addon and captured both entire
original eight-case SSR objects. Both are 1,373 bytes with zero warnings,
without duplicate own scope attributes. Their full cold DOM and hydrated
DOM also match. These bounded observations prove the dynamic scope repair;
the complete fixture driver remains failed. Its reversed CSS observation
read a 100ms `font-size` transition at `19.666px`, before the original
`18px`/`27px` counterexample settled. Its final assertions were not reached.
The supplementary Vite package tests also lacked the actual formatter
history build receipt and live preparation owner. Both failures remain
visible; no source, rule, expectation or check is removed to hide them.

The ordinary source checks found two maintenance requirements: the existing
350-line growth gate rejected the component module's five additional lines,
and the deterministic consumer migration inventory was stale. A separate
move-only commit places the complete dynamic VNode method in
`codegen/element/component/dynamic.rs`. Only its visibility changes to allow
its original caller; the complete 794-byte method and comments are kept.
The original module shrinks from 687 to 664 lines, and the new module has
30 lines. All 82 SSR library tests and seven complete SFC scope/slot controls
pass after the move; both pre-fix static module snapshots remain byte-exact.
The actual Rust generator updates the complete 19-file inventory and its
strict check passes. The existing growth limit, compiler behavior and corpus
inputs remain unchanged. Exact composed Actions and full app acceptance
are still required before queue entry.

The counterexample observer forces actual layout and awaits only the original
finite `CSSTransition.finished` promises. It retains every observed target,
property, timing, keyframe and completion or cancellation status before the
unchanged four `18px`/`27px` assertions. The whole original evaluator first
moves to a maintained helper in its own move-only commit; the original source
and CSS order, positive observations and stylesheet rules stay intact. The
exact helper passed a separate full-source official alpha.2 control, including
the actual 100ms hovered-heading transition from `20px` to `18px`/`27px`.
This local observer control does not qualify the native full app or retire
any original Actions failure.
