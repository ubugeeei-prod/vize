# VisuallyHidden behavior contract

Normative state × input → outcome table for `VisuallyHidden.vue` (`@vizejs/ui/visually-hidden`).
Every row is proven by the named test in `src/visually-hidden.test.ts`.

| #   | State           | Input  | Outcome                                                                  | Proven by                                                                 |
| --- | --------------- | ------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| V1  | slotted control | render | content stays queryable by role and accessible name, and Tab-reachable   | `keeps slotted content queryable in the accessibility tree`               |
| V2  | any             | render | exposed `element` is the rendered node                                   | `exposes the rendered element for composition`                            |
| V3  | any             | render | hidden by clipping (`clip-path`), never `display:none` (source contract) | `hides content with a recoverable clipping technique, never display:none` |
