# Grid behavior contract

Normative state x input -> outcome table for `grid.vue` (`@vizejs/ui/grid`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                                 | Proven by                                                                  |
| --- | ------------- | ----------------- | ------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| G1  | default       | resolve           | maps the default column count to one equal `fr` track and normalizes all gaps to `0`                    | `resolves a one-column grid with no authored CSS classes`                  |
| G2  | overrides     | resolve           | numeric columns become equal `fr` tracks and numeric gaps become px strings                             | `resolves numeric columns and gap overrides into native CSS grid values`   |
| G3  | invalid       | resolve           | invalid numeric columns and gaps fall back to one track and `0` gap values                              | `falls back deliberately for invalid numeric columns and gaps`             |
| G4  | default       | render            | renders a non-focusable `<div>` grid with part, data hooks, inline grid style hooks, and child slots    | `renders a non-focusable grid by default while preserving child semantics` |
| G5  | custom host   | render            | forwards semantic host attributes and renders custom track, gap, alignment, justification, and flow CSS | `renders custom tracks and auto flow on a semantic host`                   |
| G6  | any           | slot/expose       | exposes `element`, resolved columns, gaps, alignment, justification, auto flow, and native style        | `passes slot state and exposes live resolved grid state`                   |
| G7  | SSR           | isolated requests | renders byte-identical headless grid markup without request-global state                                | `renders byte-identical grid markup across isolated SSR requests`          |
| G8  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                               | `scripts/check-renderers.ts`                                               |
| G9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Grid, emit no CSS, and stay within gzip budget                   | `scripts/check-tree-shaking.mjs`                                           |

## Props

| Prop        | Type                                                            | Purpose                                                                   | Default     |
| ----------- | --------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------- |
| `as`        | `PrimitiveAs`                                                   | Native element, custom element, or component host.                        | `"div"`     |
| `columns`   | `string \| number`                                              | Native CSS `grid-template-columns`; numbers resolve to equal `fr` tracks. | `1`         |
| `gap`       | `string \| number`                                              | Native CSS `gap` value; numbers resolve to px lengths.                    | `0`         |
| `rowGap`    | `string \| number`                                              | Native CSS `row-gap` override; numbers resolve to px lengths.             | `undefined` |
| `columnGap` | `string \| number`                                              | Native CSS `column-gap` override; numbers resolve to px lengths.          | `undefined` |
| `align`     | `"stretch" \| "start" \| "center" \| "end" \| "baseline"`       | Native CSS `align-items` value for grid items.                            | `"stretch"` |
| `justify`   | `"stretch" \| "start" \| "center" \| "end"`                     | Native CSS `justify-items` value for grid items.                          | `"stretch"` |
| `autoFlow`  | `"row" \| "column" \| "dense" \| "row dense" \| "column dense"` | Native CSS `grid-auto-flow` auto-placement mode.                          | `"row"`     |

## Slots

| Slot      | Props           | Purpose                       | Default |
| --------- | --------------- | ----------------------------- | ------- |
| `default` | `GridSlotState` | Renders direct grid children. | empty   |

## Expose

| Name        | Type                  | Purpose                                      | Default                     |
| ----------- | --------------------- | -------------------------------------------- | --------------------------- |
| `element`   | `GridElement \| null` | Rendered host element or component instance. | `null`                      |
| `columns`   | `GridResolvedColumns` | Resolved CSS grid template columns value.    | `repeat(1, minmax(0, 1fr))` |
| `gap`       | `GridResolvedGap`     | Resolved CSS gap value.                      | `0`                         |
| `rowGap`    | `GridResolvedGap`     | Resolved CSS row gap value.                  | `0`                         |
| `columnGap` | `GridResolvedGap`     | Resolved CSS column gap value.               | `0`                         |
| `align`     | `GridAlign`           | Resolved `align-items` value.                | `"stretch"`                 |
| `justify`   | `GridJustify`         | Resolved `justify-items` value.              | `"stretch"`                 |
| `autoFlow`  | `GridAutoFlow`        | Resolved `grid-auto-flow` value.             | `"row"`                     |
| `style`     | `GridStyle`           | Inline native grid style hooks.              | object                      |

## Data Attributes

| Attribute        | Values                | Purpose                       | Default                     |
| ---------------- | --------------------- | ----------------------------- | --------------------------- |
| `data-vize-ui`   | `"grid"`              | Stable family selector.       | always                      |
| `data-columns`   | `GridResolvedColumns` | Resolved column track hook.   | `repeat(1, minmax(0, 1fr))` |
| `data-auto-flow` | `GridAutoFlow`        | Resolved auto-placement hook. | `"row"`                     |
| `data-align`     | `GridAlign`           | Resolved alignment hook.      | `"stretch"`                 |
| `data-justify`   | `GridJustify`         | Resolved justification hook.  | `"stretch"`                 |

## ARIA Attributes

Grid never sets `role`, `aria-hidden`, `aria-label`, `aria-labelledby`, or
`tabindex`. Consumers may pass semantic attributes to the host when the chosen
element requires them.

## CSS Custom Properties

| Property                    | Purpose                                | Default                     |
| --------------------------- | -------------------------------------- | --------------------------- |
| `--vize-ui-grid-columns`    | Value read by `grid-template-columns`. | `repeat(1, minmax(0, 1fr))` |
| `--vize-ui-grid-gap`        | Value read by `gap`.                   | `0`                         |
| `--vize-ui-grid-row-gap`    | Value read by `row-gap`.               | `0`                         |
| `--vize-ui-grid-column-gap` | Value read by `column-gap`.            | `0`                         |
| `--vize-ui-grid-align`      | Value read by `align-items`.           | `"stretch"`                 |
| `--vize-ui-grid-justify`    | Value read by `justify-items`.         | `"stretch"`                 |
| `--vize-ui-grid-auto-flow`  | Value read by `grid-auto-flow`.        | `"row"`                     |

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
