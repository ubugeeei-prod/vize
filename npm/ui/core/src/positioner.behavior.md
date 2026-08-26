# Positioner behavior contract

Normative state × input → outcome table for `positioner.vue` and
`positioner-arrow.vue` (`@vizejs/ui/positioner`). Every row is proven by the
named test in `src/positioner.test.ts`, `src/positioner-geometry.test.ts`, or
`src/positioner-ssr.test.ts`; compile-only assertions live in
`src/positioner.types.test-d.ts`.

| #   | State            | Input                                       | Outcome                                         | Proven by                                       |
| --- | ---------------- | ------------------------------------------- | ----------------------------------------------- | ----------------------------------------------- |
| P1  | unmeasured       | render without a reference                  | host stays at origin with `ready=false`         | `renders a fixed host before the first measure` |
| P2  | unmeasured       | virtual reference after mount               | coordinates move to the preferred placement     | `places below a virtual reference`              |
| P3  | overflowing side | `flip`                                      | opposite side is chosen when it overflows less  | geometry flip test                              |
| P4  | overflowing edge | `shift`                                     | floating box is clamped inside the viewport     | geometry shift test                             |
| P5  | off-screen ref   | `hide`                                      | `hidden` is true when the reference leaves view | geometry hide test                              |
| P6  | rtl              | `top-start`                                 | start alignment mirrors to the inline-end edge  | geometry rtl test                               |
| P7  | measured         | arrow child                                 | arrow is clamped along the facing edge          | `clamps the arrow along the facing edge`        |
| P8  | any              | missing Positioner provider                 | arrow setup throws a missing-context diagnostic | `rejects an arrow outside Positioner`           |
| P9  | present          | render                                      | exposed `element` is the rendered node          | `exposes the rendered element for composition`  |
| P10 | SSR              | default render                              | byte-identical origin markup, no viewport reads | SSR test                                        |
| P11 | public types     | invalid placement or mutating readonly refs | compilation rejects misuse                      | `src/positioner.types.test-d.ts`                |
