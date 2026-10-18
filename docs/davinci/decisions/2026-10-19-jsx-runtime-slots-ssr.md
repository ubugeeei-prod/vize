# Runtime JSX slots on the SSR lane

This is the SSR portion of [#6889](https://github.com/ubugeeei-prod/vize/issues/6889).
Vapor client forwarding remains unsupported and diagnosed. This does not complete
the issue's all-target requirement.

## Representation and admission

An opaque `v-slots={expression}` or `vSlots={expression}` on a JSX component
lowers to `SlotsObjectOp { value, span }`. The value is the existing borrowed
`ExprRef`, with its retained syntax, source range, and arena lifetime. It is not
a custom directive or a component prop. The ordinary L2 walker, owned dump,
expression projection, and L3 operands visit that value.

The SSR L4 lane passes the object as the third argument of `ssrRenderComponent`,
or as `createVNode` children for a dynamic component. A bare object expression
is emitted once. With authored child slots, those functions come first and the
runtime object spreads last, so its entries win collisions. The compiler does
not add a compiled-slot `_` flag beside the spread. Incoming metadata remains
part of the authored object; existing `withCtx` functions retain their owner.

The complete root must be admitted by the L4 lane. If lowering or L4 admission
fails, the existing SSR diagnostic is retained before the legacy fallback.
Built-ins with different slot ABIs, and combinations with dynamically authored
slot-template entries, remain diagnosed. Removing the front-door diagnostic
without checking the fallback would silently lose slots again.

## Primary runtime contract

The immutable test oracle is Vue and server-renderer **3.5.35**. The authored
model covers raw VNode-returning slots, compiled SSR push slots, mixed and
dynamic objects, collision order, component ownership, scope attributes,
escaping, and one evaluation. An injected `_ : 1` counterexample resolves a
component from the child instead of the slot's parent. It demonstrates why the
compiler must not manufacture that flag for a raw forwarded object.

Primary source contracts:

- [`ssrRenderComponent`](https://github.com/vuejs/core/blob/v3.5.35/packages/server-renderer/src/helpers/ssrRenderComponent.ts)
  passes `Slots | SSRSlots` through `createVNode` to the renderer.
- [`ssrRenderSlot`](https://github.com/vuejs/core/blob/v3.5.35/packages/server-renderer/src/helpers/ssrRenderSlot.ts)
  accepts either returned VNodes or pushed SSR output and passes scope IDs.
- [`componentSlots`](https://github.com/vuejs/core/blob/v3.5.35/packages/runtime-core/src/componentSlots.ts)
  normalizes raw functions using the slot object's rendering owner, while
  preserving already normalized functions.

The 3.5.42 and 3.6.0-beta.10 primary sources retain these entry contracts.
They are not byte-identical: 3.5.42 normalizes nullish slot props and changes
internal imports; beta.10 adjusts a Vapor warning condition. Source review is
not a runtime qualification for either version. The committed runtime carrier
uses 3.5.35 and records its actual version.

## Compatibility and qualification

Current V2 dumps carry `ui.slots-object`. The explicit historical V1 writer
projects only this binding to the same old `vue.directive "slots"` record,
preserving expression, escaping, and spans. The historical reader rejects the
new opcode and returns the old generic view; it grants no typed forwarding
capability. SSR L4 refuses that generic view rather than dropping its object.

The plugin document retains the old slots name/value/span tuple. Its operation
kind explicitly changes to `ui.slots-object`; this is an additive typed AST kind,
not a claim that every plugin-document field is unchanged.

Existing complete VDOM render fixtures and invalid-value/alias controls remain
in place. The existing Native SSR action also captures nine fresh JSX modules
and executes them with the pinned runtime, retaining complete HTML, warnings,
evaluation counts, calls, source, and generated modules. Those source-built
checks must run in Actions; the authored primary model alone does not establish
that Vize generated the correct modules.
