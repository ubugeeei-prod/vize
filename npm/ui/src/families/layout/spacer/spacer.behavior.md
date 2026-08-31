# Spacer behavior contract

Normative state x input -> outcome table for `spacer.vue` (`@vizejs/ui/spacer`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                           | Proven by                                                                   |
| --- | ------------- | ----------------- | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| P1  | default       | render            | renders decorative `<span aria-hidden="true">`, block-axis sizing, part, data hooks, and no focus | `renders a decorative block spacer by default`                              |
| P2  | inline axis   | render            | maps `size` to logical inline size, keeps cross-axis block size automatic, and uses inline-block  | `resolves inline and both-axis logical sizes without authored CSS classes`  |
| P3  | explicit      | render            | explicit `inlineSize`, `blockSize`, and `display` override axis-derived defaults                  | `renders explicit logical sizes on a custom host`                           |
| P4  | svg host      | render            | renders an SVG host without accessible semantics and keeps the same sizing data hooks             | `supports an SVG host without accessible content`                           |
| P5  | any           | expose            | exposes `element`, `axis`, `inlineSize`, `blockSize`, and `display` with live prop updates        | `exposes the rendered element and live resolved layout state`               |
| P6  | SSR           | isolated requests | renders byte-identical decorative logical-size markup without request-global state                | `renders byte-identical logical spacer markup across isolated SSR requests` |
| P7  | SSR svg       | render            | renders decorative SVG spacer markup with no role, label, or focus contract                       | `renders an SVG spacer without server accessibility semantics`              |
| P8  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                         | `scripts/check-renderers.ts`                                                |
| P9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Spacer, emit no CSS, and stay within gzip budget           | `scripts/check-tree-shaking.mjs`                                            |

## Props

| Prop         | Type                                                                              | Purpose                                                            | Default                                         |
| ------------ | --------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------- |
| `as`         | `PrimitiveAs`                                                                     | Native element, custom element, SVG element, or component host.    | `"span"`                                        |
| `axis`       | `"block" \| "inline" \| "both"`                                                   | Logical axis that receives `size` when explicit sizes are omitted. | `"block"`                                       |
| `size`       | `string`                                                                          | Native CSS size applied to the selected axis.                      | `"1rem"`                                        |
| `inlineSize` | `string`                                                                          | Native CSS logical inline size. Overrides axis-derived size.       | `undefined`                                     |
| `blockSize`  | `string`                                                                          | Native CSS logical block size. Overrides axis-derived size.        | `undefined`                                     |
| `display`    | `"block" \| "inline-block" \| "flex" \| "inline-flex" \| "grid" \| "inline-grid"` | CSS display mode applied to the host.                              | `"block"` for block axis, else `"inline-block"` |

## Slots

Spacer has no public slots. It is always decorative layout structure, so content
belongs in adjacent semantic elements.

## Expose

| Name         | Type                            | Purpose                                      | Default   |
| ------------ | ------------------------------- | -------------------------------------------- | --------- |
| `element`    | `SpacerElement \| null`         | Rendered host element or component instance. | `null`    |
| `axis`       | `"block" \| "inline" \| "both"` | Resolved logical axis.                       | `"block"` |
| `inlineSize` | `string`                        | Resolved logical inline size.                | `"auto"`  |
| `blockSize`  | `string`                        | Resolved logical block size.                 | `"1rem"`  |
| `display`    | `SpacerDisplay`                 | Resolved CSS display mode.                   | `"block"` |

## Data Attributes

| Attribute                      | Values                          | Purpose                       | Default   |
| ------------------------------ | ------------------------------- | ----------------------------- | --------- |
| `data-vize-ui`                 | `"spacer"`                      | Stable family selector.       | always    |
| `data-state`                   | `"sized"`                       | Spacer layout state.          | `"sized"` |
| `data-axis`                    | `"block"`, `"inline"`, `"both"` | Logical axis styling hook.    | `"block"` |
| `data-display`                 | `SpacerDisplay`                 | Applied display styling hook. | `"block"` |
| `data-vize-spacer-inline-size` | `string`                        | Resolved inline size hook.    | `"auto"`  |
| `data-vize-spacer-block-size`  | `string`                        | Resolved block size hook.     | `"1rem"`  |

## ARIA Attributes

| Attribute     | Values   | Purpose                           | Default  |
| ------------- | -------- | --------------------------------- | -------- |
| `aria-hidden` | `"true"` | Hides decorative spacing from AT. | `"true"` |

Spacer never sets `role`, `aria-label`, `aria-labelledby`, or `tabindex`.

## CSS Custom Properties

| Property                       | Purpose                               | Default  |
| ------------------------------ | ------------------------------------- | -------- |
| `--vize-ui-spacer-inline-size` | Value read by the host `inline-size`. | `"auto"` |
| `--vize-ui-spacer-block-size`  | Value read by the host `block-size`.  | `"1rem"` |

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
