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
