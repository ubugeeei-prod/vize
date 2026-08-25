# Portal behavior contract

Normative state × input → outcome table for `portal.vue` (`@vizejs/ui/portal`).
Every row is proven by the named test in `src/portal.test.ts` or
`src/portal-ssr.test.ts`.

| #   | State    | Input          | Outcome                                            | Proven by                                      |
| --- | -------- | -------------- | -------------------------------------------------- | ---------------------------------------------- |
| T1  | any      | render         | slotted content stays queryable                    | `renders slotted content`                      |
| T2  | hydrated | default target | content moves to `document.body`                   | `moves content into the document body`         |
| T3  | hydrated | `disabled`     | content stays in place                             | `keeps content in place when disabled`         |
| T4  | any      | render         | exposed `element` is the rendered node             | `exposes the rendered element for composition` |
| T5  | SSR      | default render | byte-identical in-place markup, no body relocation | SSR test                                       |
