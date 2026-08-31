# ProgressBar behavior contract

Normative state x input -> outcome table for `progress-bar.vue`
(`@vizejs/ui/progress-bar`). Every row is proven by the named mounted-DOM, SSR,
runtime-conformance, renderer, type, size, or tree-shaking gate. The legacy
`@vizejs/ui/progress` entry remains a compatibility alias for `Progress`.

| #   | State         | Input                          | Outcome                                                                                                                                        | Proven by                                                               |
| --- | ------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| P1  | any           | state normalization            | finite values clamp to `min..max`, invalid bounds repair to `min + 100`, non-finite values are indeterminate, and invalid repairs are flagged  | `normalizes determinate, complete, indeterminate, and invalid state`    |
| P2  | determinate   | render                         | renders equivalent `role="progressbar"` semantics with deterministic or explicit id, labelled name, min/max/now/value text, and no live region | `renders a named determinate progressbar with parts and CSS hooks`      |
| P3  | determinate   | render                         | exposes root, label, track, indicator, and value parts plus stable data attributes and CSS custom properties                                   | `renders a named determinate progressbar with parts and CSS hooks`      |
| P4  | indeterminate | render                         | omits `aria-valuenow`, keeps min/max semantics, mirrors `data-state="indeterminate"`, and can expose authored value text                       | `omits value semantics for indeterminate progress`                      |
| P5  | out of range  | prop update                    | ARIA attributes, data attributes, slot state, CSS hooks, and exposed state all use normalized values                                           | `clamps ARIA attributes to the safe progress range`                     |
| P6  | prop-driven   | prop update                    | slots and exposed refs/state update without local mutable progress state                                                                       | `updates slot and exposed state from props`                             |
| P7  | RTL/custom    | `dir="rtl"` and `as` component | custom hosts preserve role, deterministic label wiring, direction, parts, slots, and inline-start fill hooks                                   | `uses visible label slots, RTL direction, and consumer host components` |
| P8  | any           | Tab / render                   | remains non-interactive, does not enter sequential focus, and does not create a live region by default                                         | `does not enter the tab order or create a live region by default`       |
| P9  | SSR           | isolated requests              | renders byte-identical labelled and indeterminate server markup without request-global state                                                   | `progress-bar-ssr.test.ts`                                              |
| P10 | SSR/hydration | runtime fixture                | server markup hydrates without warnings or root node replacement                                                                               | `runtime-conformance.test.ts`                                           |
| P11 | DOM/SSR/Vapor | compile                        | authored SFC and consumer fixture compile in every renderer lane without warnings                                                              | `scripts/check-renderers.ts`                                            |
| P12 | public types  | invalid contract               | TypeScript rejects unsupported direction, state, props, and malformed readonly slot/expose shapes                                              | `src/families/feedback/progress-bar/progress-bar.types.test-d.ts`       |
| P13 | root/subpath  | consumer bundle                | root and subpath consumers retain only ProgressBar and its structural CSS within gzip budgets                                                  | `scripts/check-tree-shaking.mjs`                                        |

## Props

| Prop              | Type             | Purpose                                                                      | Default     |
| ----------------- | ---------------- | ---------------------------------------------------------------------------- | ----------- |
| `as`              | `PrimitiveAs`    | Native element, custom element, or component rendered as root.               | `"div"`     |
| `id`              | `string \| null` | Consumer-owned progressbar id.                                               | `undefined` |
| `value`           | `number \| null` | Current value; `null`, `undefined`, and non-finite values are indeterminate. | `null`      |
| `min`             | `number \| null` | Lower bound.                                                                 | `0`         |
| `max`             | `number \| null` | Upper bound; invalid bounds repair to `min + 100`.                           | `100`       |
| `dir`             | `"ltr" \| "rtl"` | Reading direction for inline-start fill and animation.                       | `"ltr"`     |
| `label`           | `string`         | Optional visible label rendered in the label part.                           | `undefined` |
| `valueLabel`      | `string`         | Optional visible value text reused as fallback `aria-valuetext`.             | `undefined` |
| `ariaLabel`       | `string`         | Accessible name when no visible label or labelledby exists.                  | `undefined` |
| `ariaLabelledby`  | `string`         | Space-separated ids that label the progressbar.                              | `undefined` |
| `ariaDescribedby` | `string`         | Space-separated ids that describe the progressbar.                           | `undefined` |
| `ariaValueText`   | `string`         | Human-readable value text; overrides `valueLabel`.                           | `undefined` |

## Slots

| Slot        | Props                  | Purpose                                        | Default           |
| ----------- | ---------------------- | ---------------------------------------------- | ----------------- |
| `default`   | `ProgressBarSlotState` | Render consumer-owned content inside the root. | none              |
| `label`     | `ProgressBarSlotState` | Render a visible label in the label part.      | `label` prop      |
| `value`     | `ProgressBarSlotState` | Render visible value text in the value part.   | `valueLabel` prop |
| `indicator` | `ProgressBarSlotState` | Render optional content inside the indicator.  | none              |

## Expose

| Name        | Type                                         | Purpose                                     | Default |
| ----------- | -------------------------------------------- | ------------------------------------------- | ------- |
| `root`      | `PrimitiveElement \| null`                   | Rendered root element or component.         | `null`  |
| `track`     | `HTMLSpanElement \| null`                    | Rendered track part.                        | `null`  |
| `indicator` | `HTMLSpanElement \| null`                    | Rendered indicator part.                    | `null`  |
| `focus`     | `(options?: FocusOptions) => void`           | Moves DOM focus to the root when supported. | n/a     |
| `value`     | `number \| null`                             | Normalized current value.                   | `null`  |
| `min`       | `number`                                     | Normalized lower bound.                     | `0`     |
| `max`       | `number`                                     | Normalized upper bound.                     | `100`   |
| `percent`   | `number \| null`                             | Completion percentage.                      | `null`  |
| `ratio`     | `number \| null`                             | Completion ratio from 0 to 1.               | `null`  |
| `dir`       | `"ltr" \| "rtl"`                             | Reflected reading direction.                | `"ltr"` |
| `state`     | `"loading" \| "complete" \| "indeterminate"` | Stable progress state.                      | derived |
| `style`     | `ProgressBarStyle`                           | CSS custom property hooks applied to root.  | derived |

## Data Attributes

| Attribute            | Host  | Values                                                                                             | Purpose                 | Default                    |
| -------------------- | ----- | -------------------------------------------------------------------------------------------------- | ----------------------- | -------------------------- |
| `data-vize-ui`       | root  | `"progress-bar"`                                                                                   | Stable family selector. | always                     |
| `data-vize-ui`       | parts | `"progress-bar-label"`, `"progress-bar-track"`, `"progress-bar-indicator"`, `"progress-bar-value"` | Stable part selectors.  | derived                    |
| `data-dir`           | root  | `"ltr"`, `"rtl"`                                                                                   | Direction hook.         | `"ltr"`                    |
| `data-state`         | root  | `"loading"`, `"complete"`, `"indeterminate"`                                                       | Stable state hook.      | derived                    |
| `data-labelled`      | root  | `"true"`, `"false"`                                                                                | Accessible-name hook.   | derived                    |
| `data-indeterminate` | root  | `"true"`, `"false"`                                                                                | Value policy hook.      | derived                    |
| `data-complete`      | root  | `"true"`, `"false"`                                                                                | Completion hook.        | derived                    |
| `data-invalid`       | root  | `"true"`                                                                                           | Raw-input repair hook.  | omitted                    |
| `data-value`         | root  | `number`                                                                                           | Normalized value.       | omitted when indeterminate |
| `data-min`           | root  | `number`                                                                                           | Normalized lower bound. | derived                    |
| `data-max`           | root  | `number`                                                                                           | Normalized upper bound. | derived                    |
| `data-percent`       | root  | `number`                                                                                           | Completion percentage.  | omitted when indeterminate |

## CSS Custom Properties

| Property                                        | Host      | Purpose                           | Default             |
| ----------------------------------------------- | --------- | --------------------------------- | ------------------- |
| `--vize-ui-progress-bar-min`                    | root      | Normalized lower bound.           | derived             |
| `--vize-ui-progress-bar-max`                    | root      | Normalized upper bound.           | derived             |
| `--vize-ui-progress-bar-value`                  | root      | Normalized value or min.          | derived             |
| `--vize-ui-progress-bar-percent`                | root      | Completion percentage with `%`.   | derived             |
| `--vize-ui-progress-bar-ratio`                  | root      | Completion ratio.                 | derived             |
| `--vize-ui-progress-bar-inline-size`            | root      | Root inline size.                 | `100%`              |
| `--vize-ui-progress-bar-gap`                    | root      | Root grid gap.                    | `0.25rem`           |
| `--vize-ui-progress-bar-track-block-size`       | track     | Track block size.                 | `0.5rem`            |
| `--vize-ui-progress-bar-track-radius`           | track     | Track and indicator radius.       | `999px`             |
| `--vize-ui-progress-bar-track-color`            | track     | Track background.                 | mixed current color |
| `--vize-ui-progress-bar-indicator-color`        | indicator | Indicator background.             | `currentColor`      |
| `--vize-ui-progress-bar-duration`               | indicator | Determinate transition duration.  | `160ms`             |
| `--vize-ui-progress-bar-indeterminate-size`     | indicator | Indeterminate indicator width.    | `40%`               |
| `--vize-ui-progress-bar-indeterminate-duration` | indicator | Indeterminate animation duration. | `1.2s`              |

## Parts

| Part        | Purpose                              | Default                |
| ----------- | ------------------------------------ | ---------------------- |
| `root`      | Progressbar semantics and CSS hooks. | always                 |
| `label`     | Visible label container.             | when labelled          |
| `track`     | Structural track.                    | always                 |
| `indicator` | Filled or indeterminate indicator.   | always                 |
| `value`     | Visible value text container.        | when value text exists |
