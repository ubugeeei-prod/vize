# VisuallyHidden behavior contract

Normative state × input → outcome table for `visually-hidden.vue` (`@vizejs/ui/visually-hidden`).
Rows V1–V3 are proven by the named test in
`src/families/accessibility/visually-hidden/visually-hidden.test.ts`; V4 is
proven on the packaged stylesheet by `src/families/foundations/theme/style-pipeline.test.ts`.

| #   | State           | Input          | Outcome                                                                    | Proven by                                                                                |
| --- | --------------- | -------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| V1  | slotted control | render         | content stays queryable by role and accessible name, and Tab-reachable     | `keeps slotted content queryable in the accessibility tree`                              |
| V2  | any             | render         | exposed `element` is the rendered node                                     | `exposes the rendered element for composition`                                           |
| V3  | any             | render         | hidden by clipping (`clip-path`), never `display:none` (source contract)   | `hides content with a recoverable clipping technique, never display:none`                |
| V4  | slotted control | focus moves in | content stays clipped; revealing on focus is a different, opt-in component | `authored nesting, layers, logical properties, and color functions compile to the floor` |

## CSS custom properties

| Custom property                        | Purpose                                                      | Default              |
| -------------------------------------- | ------------------------------------------------------------ | -------------------- |
| `--vize-ui-visually-hidden-background` | Repaint the clipped box (for example while debugging layout) | `oklch(0% 0 0 / 0%)` |
