# Forwarding behavior contract

Normative state x input -> outcome table for `@vizejs/ui/forwarding`. Every row
is proven by `src/families/foundations/forwarding/forwarding.test.ts`;
compile-only assertions live in
`src/families/foundations/forwarding/forwarding.types.test-d.ts`.

| #   | State             | Input                             | Outcome                                                                              | Proven by                                                               |
| --- | ----------------- | --------------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| FW1 | event name        | `toHandlerKey(name)`              | Vue's handler key (`onUpdate:modelValue`, `onValueChange`)                           | `converts event names to Vue handler keys`                              |
| FW2 | selected events   | `useEmitAsProps(emit, names)`     | frozen `onX` props re-emit with the original payload                                 | `re-emits selected events with their payloads`                          |
| FW3 | reactive props    | `useForwardProps(props)`          | only defined props are forwarded, reactively, so child defaults survive              | `forwards only defined props so child defaults survive`                 |
| FW4 | wrapper component | click, then call exposed method   | props and emits reach the child and parent; the child's exposed API is reachable     | `wrappers forward props, emits, and the child's exposed API in the DOM` |
| FW5 | forwarded element | before and after `forwardRef(el)` | properties are `undefined` before mount; `$el` and bound element methods after       | `forwarded expose binds element methods and is empty before mount`      |
| FW6 | server render     | render twice, hydrate, click      | byte-identical markup, no hydration diagnostics, interactive after hydration         | `renders identical wrapper markup on the server and hydrates cleanly`   |
| FW7 | public type API   | `defineEmits` emit, props, expose | event names/payloads inferred from `emit`; undeclared events and wrong payloads fail | `src/families/foundations/forwarding/forwarding.types.test-d.ts`        |

All helpers are instance-free (no `getCurrentInstance()`), so they work in
Vapor components and plain effect scopes.
