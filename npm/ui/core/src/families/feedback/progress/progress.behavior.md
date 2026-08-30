# Progress behavior contract

Normative state/input -> outcome table for `progress.vue`
(`@vizejs/ui/progress`). Every row is proven by the named mounted-DOM, SSR,
runtime-conformance, renderer, type, size, or tree-shaking gate.

| #   | State         | Input               | Outcome                                                                                                        | Proven by                                                         |
| --- | ------------- | ------------------- | -------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| P1  | any           | state normalization | finite determinate values clamp to `0..max`; invalid maximum values fall back to `100`; non-finite is unknown  | `normalizes determinate, complete, and indeterminate state`       |
| P2  | determinate   | render              | native `<progress>`, implicit `progressbar`, deterministic or explicit `id`, accessible name, and value attrs  | `renders a named native determinate progressbar`                  |
| P3  | determinate   | render              | exposes `data-state`, `data-value`, `data-max`, `data-percent`, `data-complete`, and `part="root"`             | `renders a named native determinate progressbar`                  |
| P4  | indeterminate | render              | omits the native `value` attribute and sets `data-state="indeterminate"`                                       | `omits the native value for indeterminate progress`               |
| P5  | out of range  | prop update         | native attributes, DOM properties, data attributes, slot state, and exposed state all use the normalized value | `clamps native attributes to the safe progress range`             |
| P6  | prop-driven   | prop update         | slot props and exposed state update without local mutable progress state                                       | `updates slot and exposed state from props`                       |
| P7  | any           | Tab / render        | remains non-interactive, does not enter sequential focus, and does not create a live region by default         | `does not enter the tab order or create a live region by default` |
| P8  | SSR           | isolated requests   | server markup is byte-stable and keeps native progress semantics                                               | `progress-ssr.test.ts`                                            |
| P9  | SSR/hydration | runtime fixture     | server markup is stable and hydrates without warnings or node replacement                                      | `runtime-conformance.test.ts`                                     |
| P10 | public types  | invalid state       | TypeScript rejects closed-contract misuse                                                                      | `src/families/feedback/progress/progress.types.test-d.ts`         |

The Progress primitive is headless. It ships no CSS, exposes no live-region
policy by default, and relies on the native `progress` element for platform
semantics. Consumers that need spoken milestones should compose Progress with
`@vizejs/ui/announcer` and coalesce updates there.
