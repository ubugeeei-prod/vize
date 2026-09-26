# JSX falsy logical-and children

Tracked in [#6887](https://github.com/ubugeeei-prod/vize/issues/6887), with the
broader value-sensitive oracles in [#6890](https://github.com/ubugeeei-prod/vize/issues/6890).

## Legacy behavior fix

`count && <X/>` evaluates to `count` when it is falsy. The existing native
lowering loses `0`, `-0`, `0n` and `NaN` by always producing a one-branch If.
This change preserves those JSX child values, including their reactive updates.

- Plain If is retained only for runtime booleans proven by syntax or an
  immutable const initializer resolved through OXC symbol IDs. Comparisons,
  logical-not, boolean literals and boolean compositions qualify. TypeScript
  assertions, mutable bindings, unknown calls and property reads do not.
- Unknown conditions reuse the existing lexical binding scope. VDOM uses one
  IIFE evaluation, SSR one block evaluation, and Vapor a computed source.
  This is not a list: it introduces no loop, array or list fragment.
- The truthy branch renders the actual RHS JSX. Arrays, VNodes and boxed
  booleans are truthy and never enter text interpolation.
- The falsy branch renders only number, string or bigint primitives. False,
  null and undefined retain the backend's existing empty comment behavior.
- Text-only If branches need the same Fragment markers in SSR as their
  actual client VDOM. The SSR correction is limited to the direct single
  Text/Interpolation branch shape. It adds no artificial children.

## Evidence and acceptance

The regression fixtures live in `tests/_fixtures/differential/jsx/` and are
executed by the Rust `falsy_and` integration test through the real pinned Vue
runtime and `@vue/babel-plugin-jsx`. Complete compiler code, diagnostics, maps,
raw HTML/comments, DOM trees, evaluation counts, hydration warnings and keyed
node identity are separate observations. Byte comparisons never use trimmed
insta snapshots or a normalized rendering in place of the complete output.

The public 0.429.0 compiler reproduces missing `0` and `NaN` under Vue 3.5.43
and Babel JSX 2.0.1. It also reproduces a real hydration warning for a ternary
whose alternate is a text value: client Fragment markers are absent in SSR.
The mounted regression targets the project's existing Vue Vapor RC runtime.

This is a legacy correction, not a Davinci native acceptance claim. The L2
projection refuses a synthetic lexical scope instead of treating it as ui.for;
the existing default VDOM/SSR selection then uses the scope-capable legacy
backend. The boolean fast path keeps its existing native If projection.

## Remaining work

- Register a complete JSX adapter in the shared differential product registry.
  Its current compiler adapter owns only its exact SFC profiles; JSX fixtures
  do not enter the global Vue fixture walker.
- Record a fresh source build and run all affected compiler/runtime gates.
- Add the native JSX scope/value producer and compare its complete output
  byte-for-byte with the fixed legacy result. Native handled/equivalent credit
  remains zero for the unknown-value fixtures until that producer pair exists.
- Keep #6887 open until both the legacy fix and native acceptance are complete;
  #6890 also remains open for the broader dialect/value matrix.
