# Portal behavior contract

Normative state × input → outcome table for `portal.vue` and `portal-stack.ts`
(`@vizejs/ui/portal`). Every row is proven by the named test in
`src/families/overlays/portal/portal.test.ts`,
`src/families/overlays/portal/portal-stack.test.ts`, or
`src/families/overlays/portal/portal-ssr.test.ts`; compile-only assertions
live in `src/families/overlays/portal/portal.types.test-d.ts`.

| #   | State        | Input                | Outcome                                              | Proven by                                                        |
| --- | ------------ | -------------------- | ---------------------------------------------------- | ---------------------------------------------------------------- |
| T1  | any          | render               | slotted content stays queryable                      | `renders slotted content`                                        |
| T2  | hydrated     | default target       | content moves to `document.body`                     | `moves content into the document body`                           |
| T3  | hydrated     | `disabled`           | content stays in place                               | `keeps content in place when disabled`                           |
| T4  | any          | render               | exposed `element` is the rendered node               | `exposes the rendered element for composition`                   |
| T5  | SSR          | default render       | byte-identical in-place markup, no body relocation   | SSR test                                                         |
| T6  | nested       | portal inside portal | layers publish `data-vize-portal-depth` 0, 1, …      | `publishes incrementing depth for nested portals`                |
| T7  | mounted      | stack query          | shared stack orders layers shallow-to-deep, top last | `tracks nested layers shallow-to-deep in the shared stack`       |
| T8  | unmounting   | layer leaves         | released layers drop off the stack immediately       | `releases layers from the stack on unmount`                      |
| T9  | SSR          | nested render        | deterministic depth markup, stack stays untouched    | `renders deterministic nesting depth without touching the stack` |
| T10 | public types | mutating stack entry | compilation rejects misuse                           | `src/families/overlays/portal/portal.types.test-d.ts`            |
