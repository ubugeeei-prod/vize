# Interaction Hooks Behavior

| Contract                | Expected behavior                                                                                                                            | Evidence                            |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| SSR setup               | `useInteractionHooks` does not read DOM globals during setup and renders deterministic HTML without serialized event handlers.               | `interaction-hooks-ssr.test.ts`     |
| Feature composition     | Press, hover, focus-ring, and focus-within props are merged into one immutable host prop object without changing the underlying controllers. | `interaction-hooks.test.ts`         |
| Handler fan-out         | Features that share a native handler, such as press and shortcuts on `keydown`, both receive the event in deterministic order.               | `interaction-hooks.test.ts`         |
| Input modality          | Optional modality tracking reports keyboard, pointer, touch, and virtual intent without requiring component authors to wire document state.  | `interaction-hooks-ssr.test.ts`     |
| Keyboard shortcuts      | Optional shortcut routing stays host-scoped, avoids retaining the full shortcut family, and preserves literal controller typing.             | `interaction-hooks.types.test-d.ts` |
| Vue renderer fixture    | `interaction-hooks-example.vue` binds the composed props to one host and reflects active interaction state without replacing SSR markup.     | `interaction-hooks.test.ts`         |
| Literal feature removal | Passing literal `false` for a feature returns `null` for that controller and removes that feature's props from the inferred prop type.       | `interaction-hooks.types.test-d.ts` |
| Shared disabled state   | Top-level `isDisabled` flows to each enabled feature while per-feature options may still override it.                                        | `interaction-hooks.types.test-d.ts` |
| Lifecycle ownership     | `cancel` and `dispose` fan out to every enabled controller and aggregate cleanup failures rather than dropping them.                         | `interaction-hooks.test.ts`         |
