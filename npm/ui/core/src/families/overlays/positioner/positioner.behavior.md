# Positioner behavior contract

Normative state × input → outcome table for `positioner.vue` and
`positioner-arrow.vue` (`@vizejs/ui/positioner`). Every row is proven by the
named test in `src/families/overlays/positioner/positioner.test.ts`,
`src/families/overlays/positioner/positioner-geometry.test.ts`,
`src/families/overlays/positioner/positioner-size.test.ts`,
`src/families/overlays/positioner/positioner-viewport.test.ts`, or
`src/families/overlays/positioner/positioner-ssr.test.ts`; compile-only
assertions live in `src/families/overlays/positioner/positioner.types.test-d.ts`.

| #   | State            | Input                                       | Outcome                                              | Proven by                                                           |
| --- | ---------------- | ------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------- |
| P1  | unmeasured       | render without a reference                  | host stays at origin with `ready=false`              | `renders a fixed host before the first measure`                     |
| P2  | unmeasured       | virtual reference after mount               | coordinates move to the preferred placement          | `places below a virtual reference`                                  |
| P3  | overflowing side | `flip`                                      | opposite side is chosen when it overflows less       | geometry flip test                                                  |
| P4  | overflowing edge | `shift`                                     | floating box is clamped inside the viewport          | geometry shift test                                                 |
| P5  | off-screen ref   | `hide`                                      | `hidden` is true when the reference leaves view      | geometry hide test                                                  |
| P6  | rtl              | `top-start`                                 | start alignment mirrors to the inline-end edge       | geometry rtl test                                                   |
| P7  | measured         | arrow child                                 | arrow is clamped along the facing edge               | `clamps the arrow along the facing edge`                            |
| P8  | any              | missing Positioner provider                 | arrow setup throws a missing-context diagnostic      | `rejects an arrow outside Positioner`                               |
| P9  | present          | render                                      | exposed `element` is the rendered node               | `exposes the rendered element for composition`                      |
| P10 | SSR              | default render                              | byte-identical origin markup, no viewport reads      | SSR test                                                            |
| P11 | public types     | invalid placement or mutating readonly refs | compilation rejects misuse                           | `src/families/overlays/positioner/positioner.types.test-d.ts`       |
| P12 | measured         | any side                                    | available space stops at the reference edge          | `measures available space on every side`                            |
| P13 | measured         | `size`                                      | host publishes max size and custom properties        | `constrains the host to the available space when size is enabled`   |
| P14 | measured         | default `size`                              | host style stays byte-identical without opt-in       | `leaves the host style untouched when size is off`                  |
| P15 | keyboard open    | visual viewport shrinks                     | floating box stays inside the visible viewport       | `keeps the floating box inside the keyboard-shrunk visual viewport` |
| P16 | notched display  | `safeArea`                                  | collision handling respects `env(safe-area-inset-*)` | `applies safe-area insets to collision handling`                    |
| P17 | any              | viewport insets                             | inset math clamps at an empty box                    | `insets the viewport by per-edge insets`                            |
