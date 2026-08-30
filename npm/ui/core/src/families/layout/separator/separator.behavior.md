# Separator behavior contract

Normative state x input -> outcome table for `separator.vue` (`@vizejs/ui/separator`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State          | Input             | Outcome                                                                                         | Proven by                                                                       |
| --- | -------------- | ----------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| S1  | default        | render            | renders native `<hr role="separator">`, horizontal orientation, stable data hooks, and no focus | `renders a native horizontal separator by default`                              |
| S2  | vertical       | render            | renders the requested host with `aria-orientation="vertical"` and a consumer-owned label        | `renders a labelled vertical separator on a custom host`                        |
| S3  | decorative     | render            | uses `role="presentation"`, hides from the accessibility tree, and suppresses labels            | `decorative separators opt out of accessibility semantics`                      |
| S4  | any            | expose            | exposes `element`, `orientation`, and `decorative` with live prop updates                       | `exposes the rendered element and live separator state`                         |
| S5  | SSR semantic   | isolated requests | renders byte-identical semantic separator markup without request-global state                   | `renders byte-identical semantic separator markup across isolated SSR requests` |
| S6  | SSR decorative | render            | renders decorative markup without separator ARIA or labels                                      | `renders decorative server markup without semantic ARIA`                        |
| S7  | DOM/SSR/Vapor  | compile           | authored SFC compiles in every renderer lane without warnings or fallback                       | `scripts/check-renderers.ts`                                                    |
| S8  | root/subpath   | consumer bundle   | root and subpath consumers retain only Separator, emit no CSS, and stay within gzip budget      | `scripts/check-tree-shaking.mjs`                                                |

## Props

| Prop             | Type                         | Purpose                                                        | Default        |
| ---------------- | ---------------------------- | -------------------------------------------------------------- | -------------- |
| `as`             | `PrimitiveAs`                | Native element, custom element, or component rendered as host. | `"hr"`         |
| `orientation`    | `"horizontal" \| "vertical"` | Logical separator axis announced to assistive technology.      | `"horizontal"` |
| `decorative`     | `boolean`                    | Hide the separator from assistive technology when visual only. | `false`        |
| `ariaLabel`      | `string`                     | Accessible name for a semantic separator. Ignored decorative.  | `undefined`    |
| `ariaLabelledby` | `string`                     | Space-separated ids that label a semantic separator.           | `undefined`    |

## Slots

Separator has no public slots. The default host is native `<hr>`, a void element,
so content belongs in adjacent labelled regions rather than inside the separator.

## Expose

| Name          | Type                         | Purpose                                           | Default        |
| ------------- | ---------------------------- | ------------------------------------------------- | -------------- |
| `element`     | `SeparatorElement \| null`   | Rendered host element or component instance.      | `null`         |
| `orientation` | `"horizontal" \| "vertical"` | Logical axis used by ARIA and `data-orientation`. | `"horizontal"` |
| `decorative`  | `boolean`                    | Whether accessibility semantics are suppressed.   | `false`        |

## Data Attributes

| Attribute          | Values                       | Purpose                        | Default        |
| ------------------ | ---------------------------- | ------------------------------ | -------------- |
| `data-vize-ui`     | `"separator"`                | Stable family selector.        | always         |
| `data-state`       | `"semantic"`, `"decorative"` | Accessibility semantics state. | `"semantic"`   |
| `data-orientation` | `"horizontal"`, `"vertical"` | Logical axis styling hook.     | `"horizontal"` |

## ARIA Attributes

| Attribute          | Values                          | Purpose                                      | Default        |
| ------------------ | ------------------------------- | -------------------------------------------- | -------------- |
| `role`             | `"separator"`, `"presentation"` | Semantic separator or decorative opt-out.    | `"separator"`  |
| `aria-orientation` | `"horizontal"`, `"vertical"`    | Axis announced for semantic separators.      | `"horizontal"` |
| `aria-hidden`      | `"true"`                        | Hides decorative separators from AT.         | `undefined`    |
| `aria-label`       | `string`                        | Optional semantic separator name.            | `undefined`    |
| `aria-labelledby`  | `string`                        | Optional semantic separator labelled-by ids. | `undefined`    |

## CSS Custom Properties

Separator defines no CSS custom properties and ships no stylesheet. Consumers own
visual thickness, color, spacing, and vertical sizing through ordinary CSS.

## Parts

Separator defines no `part` names because it renders a single native host.
