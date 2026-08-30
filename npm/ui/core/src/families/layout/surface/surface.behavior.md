# Surface behavior contract

Normative state x input -> outcome table for `surface.vue` (`@vizejs/ui/surface`).
Every row is proven by the named mounted-DOM, SSR, runtime-conformance, or
packaging test. A row without a passing test is a contract violation.

| #   | State            | Input             | Outcome                                                                                                         | Proven by                                                                          |
| --- | ---------------- | ----------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| S1  | default          | render / Tab      | renders `<section data-vize-ui="surface">`, `part="root"`, no visual hooks, no ARIA, and no focus target        | `renders a section surface by default without visual, focus, or ARIA side effects` |
| S2  | semantic hosts   | render            | renders the documented `section`, `article`, `aside`, and `div` examples while preserving native semantics      | `renders every supported semantic host with optional hooks`                        |
| S3  | ARIA IDREF props | render            | normalizes typed `ariaLabelledby` and `ariaDescribedby` into native `aria-labelledby` and `aria-describedby`    | `normalizes typed ARIA ID references and preserves ordinary fallthrough attrs`     |
| S4  | consumer attrs   | fallthrough attrs | preserves consumer-owned id, role, label, tabindex, class, style, and data attributes without deriving defaults | `normalizes typed ARIA ID references and preserves ordinary fallthrough attrs`     |
| S5  | optional hooks   | render            | mirrors `tone` and `elevation` only when provided; default markup omits both attributes and ships no CSS        | `renders a section surface by default without visual, focus, or ARIA side effects` |
| S6  | any              | slot/expose       | passes semantic, ARIA, tone, elevation, labelled, and described state to the slot and exposes live public state | `passes slot state and exposes live surface state`                                 |
| S7  | custom host      | render            | renders consumer components while preserving Surface data hooks, ARIA, and ordinary fallthrough attributes      | `renders a consumer component host without dropping surface hooks`                 |
| S8  | SSR labelled     | isolated requests | renders byte-identical labelled server markup without request-global state                                      | `renders byte-identical labelled surface markup across isolated SSR requests`      |
| S9  | SSR default      | render            | omits optional ARIA and styling hooks from default server markup                                                | `omits optional ARIA and data hooks from default SSR markup`                       |
| S10 | SSR/hydration    | runtime fixture   | server markup hydrates without warnings or root node replacement                                                | `runtime-conformance.test.ts`                                                      |
| S11 | DOM/SSR/Vapor    | compile           | authored SFC compiles in every renderer lane without warnings or fallback                                       | `scripts/check-renderers.ts`                                                       |
| S12 | root/subpath     | consumer bundle   | root and subpath consumers retain only Surface, emit no CSS, and stay within gzip budget                        | `scripts/check-tree-shaking.mjs`                                                   |

## Props

| Prop              | Type                                                                               | Purpose                                                                 | Default     |
| ----------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ----------- |
| `as`              | `PrimitiveAs`                                                                      | Native element, custom element, or component rendered by the primitive. | `"section"` |
| `ariaLabelledby`  | `string`                                                                           | Space-separated ids rendered as `aria-labelledby`.                      | `undefined` |
| `ariaDescribedby` | `string`                                                                           | Space-separated ids rendered as `aria-describedby`.                     | `undefined` |
| `tone`            | `"neutral" \| "muted" \| "accent" \| "info" \| "success" \| "warning" \| "danger"` | Optional consumer styling hook mirrored to `data-tone`.                 | `undefined` |
| `elevation`       | `"raised" \| "overlay" \| "floating"`                                              | Optional consumer elevation hook mirrored to `data-elevation`.          | `undefined` |

## Slots

| Slot      | Props                                                                                                                                                           | Purpose                                | Default |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- | ------- |
| `default` | `{ as: SurfaceAs; ariaLabelledby?: string; ariaDescribedby?: string; tone?: SurfaceTone; elevation?: SurfaceElevation; labelled: boolean; described: boolean }` | Render consumer-owned surface content. | none    |

## Expose

| Name              | Type                            | Purpose                                  | Default     |
| ----------------- | ------------------------------- | ---------------------------------------- | ----------- |
| `element`         | `PrimitiveElement \| null`      | Rendered host element or component.      | `null`      |
| `as`              | `SurfaceAs`                     | Rendered semantic host.                  | `"section"` |
| `ariaLabelledby`  | `string \| undefined`           | Normalized labeling IDREF list.          | `undefined` |
| `ariaDescribedby` | `string \| undefined`           | Normalized description IDREF list.       | `undefined` |
| `tone`            | `SurfaceTone \| undefined`      | Optional consumer tone hook.             | `undefined` |
| `elevation`       | `SurfaceElevation \| undefined` | Optional consumer elevation hook.        | `undefined` |
| `labelled`        | `boolean`                       | Whether a labeling IDREF is rendered.    | `false`     |
| `described`       | `boolean`                       | Whether a description IDREF is rendered. | `false`     |

## Data Attributes

| Attribute        | Values                                | Purpose                          | Default     |
| ---------------- | ------------------------------------- | -------------------------------- | ----------- |
| `data-vize-ui`   | `"surface"`                           | Stable family selector.          | always      |
| `data-tone`      | `SurfaceTone`                         | Consumer tone styling hook.      | `undefined` |
| `data-elevation` | `"raised"`, `"overlay"`, `"floating"` | Consumer elevation styling hook. | `undefined` |

## ARIA Attributes

Surface renders `aria-labelledby` and `aria-describedby` only from the typed
`ariaLabelledby` and `ariaDescribedby` props after whitespace normalization.
It never generates ids, roles, `tabindex`, `aria-hidden`, `aria-live`, or
accessible names. Consumers may still pass ordinary fallthrough attributes
for specialized region, navigation, or form-group semantics.

## CSS Custom Properties

Surface defines no CSS custom properties and ships no stylesheet. Consumers own
shape, border, spacing, color, shadow, backdrop, density, and responsive
treatment through ordinary CSS or opt-in Vize theme presets.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
