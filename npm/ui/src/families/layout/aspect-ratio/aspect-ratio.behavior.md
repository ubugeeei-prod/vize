# AspectRatio behavior contract

Normative state x input -> outcome table for `aspect-ratio.vue` (`@vizejs/ui/aspect-ratio`).
Every row is proven by the named mounted-DOM test, SSR test, runtime
conformance check, or validation script. A row without a passing test or check
is a contract violation.

| #   | State             | Input             | Outcome                                                                                      | Proven by                                                                    |
| --- | ----------------- | ----------------- | -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| A1  | default           | render            | renders `<div data-vize-ui="aspect-ratio">`, `data-state="valid"`, and a square ratio        | `renders a square headless host by default`                                  |
| A2  | valid ratio       | render            | publishes the normalized ratio to `data-vize-aspect-ratio` and `--vize-ui-aspect-ratio`      | `publishes the requested ratio through stable data and style hooks`          |
| A3  | invalid ratio     | render            | falls back to ratio `1` and `data-state="fallback"` for non-positive or non-finite values    | `falls back deliberately for non-positive and non-finite ratios`             |
| A4  | `as="section"`    | render            | renders the requested semantic host and passes normalized `ratio` and `invalid` to the slot  | `renders a semantic host and exposes normalized slot state`                  |
| A5  | any               | expose            | exposes `element`, normalized `ratio`, and `invalid`                                         | `exposes the rendered element and live normalized ratio state`               |
| A6  | SSR valid ratio   | isolated requests | renders byte-identical intrinsic-ratio markup with the same data and style hooks             | `renders byte-identical intrinsic ratio markup across isolated SSR requests` |
| A7  | SSR invalid ratio | render            | renders fallback ratio markup without request-global state                                   | `renders fallback ratio markup for invalid server input`                     |
| A8  | DOM/SSR/Vapor     | compile           | authored SFC compiles in every renderer lane without warnings or fallback                    | `scripts/check-renderers.ts`                                                 |
| A9  | root/subpath      | consumer bundle   | root and subpath consumers retain only AspectRatio, emit no CSS, and stay within gzip budget | `scripts/check-tree-shaking.mjs`                                             |

## Props

| Prop    | Type          | Purpose                                                        | Default |
| ------- | ------------- | -------------------------------------------------------------- | ------- |
| `as`    | `PrimitiveAs` | Native element, custom element, or component rendered as host. | `"div"` |
| `ratio` | `number`      | Positive finite width divided by height. Invalid values use 1. | `1`     |

## Slots

| Slot      | Props                                 | Purpose                               | Default |
| --------- | ------------------------------------- | ------------------------------------- | ------- |
| `default` | `{ ratio: number; invalid: boolean }` | Render content inside the ratio host. | none    |

## Expose

| Name      | Type                         | Purpose                                              | Default |
| --------- | ---------------------------- | ---------------------------------------------------- | ------- |
| `element` | `AspectRatioElement \| null` | Rendered host element or component instance.         | `null`  |
| `ratio`   | `number`                     | Ratio used for the rendered box after validation.    | `1`     |
| `invalid` | `boolean`                    | Whether the provided ratio fell back to the default. | `false` |

## Data Attributes

| Attribute                | Values                  | Purpose                                  | Default   |
| ------------------------ | ----------------------- | ---------------------------------------- | --------- |
| `data-vize-ui`           | `"aspect-ratio"`        | Stable family selector.                  | always    |
| `data-state`             | `"valid"`, `"fallback"` | Ratio validation state.                  | `"valid"` |
| `data-vize-aspect-ratio` | positive number string  | Normalized ratio used by the host style. | `"1"`     |

## CSS Custom Properties

| Custom property          | Purpose                                              | Default |
| ------------------------ | ---------------------------------------------------- | ------- |
| `--vize-ui-aspect-ratio` | Value read by the inline `aspect-ratio` declaration. | `"1"`   |

## Styling Contract

AspectRatio is headless: it emits no stylesheet and no visual preset. The host
receives only the inline `aspect-ratio: var(--vize-ui-aspect-ratio)` declaration
and matching custom property required for intrinsic layout.
