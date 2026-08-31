# Stack behavior contract

Normative state x input -> outcome table for `stack.vue` (`@vizejs/ui/stack`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                  | Proven by                                                                         |
| --- | ------------- | ----------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| P1  | default       | render            | renders a non-focusable `<div>` flex stack with part, data hooks, default gap, and slots | `renders a non-focusable block stack by default while preserving child semantics` |
| P2  | block axis    | resolve           | maps logical block flow to `flex-direction: column` with no authored wrapping behavior   | `resolves a logical block stack without authored wrapping behavior`               |
| P3  | inline axis   | resolve           | maps logical inline reverse flow to `row-reverse` and preserves native logical values    | `resolves reversed inline flow with native logical alignment values`              |
| P4  | custom host   | render            | forwards host attributes such as `dir` and renders RTL-aware inline stack data hooks     | `renders an RTL-aware inline stack on a custom host`                              |
| P5  | any           | slot/expose       | exposes `element`, `axis`, `reversed`, `direction`, `gap`, `align`, `justify`, and state | `passes slot state and exposes live resolved layout state`                        |
| P6  | SSR           | isolated requests | renders byte-identical flex stack markup without request-global state                    | `renders byte-identical logical stack markup across isolated SSR requests`        |
| P7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                | `scripts/check-renderers.ts`                                                      |
| P8  | root/subpath  | consumer bundle   | root and subpath consumers retain only Stack, emit no CSS, and stay within gzip budget   | `scripts/check-tree-shaking.mjs`                                                  |

## Props

| Prop       | Type                                                                                  | Purpose                                                            | Default     |
| ---------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ----------- |
| `as`       | `PrimitiveAs`                                                                         | Native element, custom element, or component host.                 | `"div"`     |
| `axis`     | `"block" \| "inline"`                                                                 | Logical flex main axis.                                            | `"block"`   |
| `reversed` | `boolean`                                                                             | Reverse the selected logical main axis without changing DOM order. | `false`     |
| `gap`      | `string`                                                                              | Native CSS `gap` value between direct children.                    | `"1rem"`    |
| `align`    | `"stretch" \| "start" \| "center" \| "end" \| "baseline"`                             | Native CSS `align-items` value for the cross axis.                 | `"stretch"` |
| `justify`  | `"start" \| "center" \| "end" \| "space-between" \| "space-around" \| "space-evenly"` | Native CSS `justify-content` value for the main axis.              | `"start"`   |

## Slots

| Slot      | Props            | Purpose                        | Default |
| --------- | ---------------- | ------------------------------ | ------- |
| `default` | `StackSlotState` | Renders direct stack children. | empty   |

## Expose

| Name        | Type                   | Purpose                                      | Default     |
| ----------- | ---------------------- | -------------------------------------------- | ----------- |
| `element`   | `StackElement \| null` | Rendered host element or component instance. | `null`      |
| `axis`      | `"block" \| "inline"`  | Resolved logical axis.                       | `"block"`   |
| `reversed`  | `boolean`              | Whether the resolved axis is reversed.       | `false`     |
| `direction` | `StackFlexDirection`   | Resolved CSS flex direction.                 | `"column"`  |
| `gap`       | `string`               | Resolved CSS gap value.                      | `"1rem"`    |
| `align`     | `StackAlign`           | Resolved cross-axis alignment.               | `"stretch"` |
| `justify`   | `StackJustify`         | Resolved main-axis distribution.             | `"start"`   |
| `state`     | `"stacked"`            | Stable layout state token.                   | `"stacked"` |

## Data Attributes

| Attribute                   | Values                | Purpose                       | Default     |
| --------------------------- | --------------------- | ----------------------------- | ----------- |
| `data-vize-ui`              | `"stack"`             | Stable family selector.       | always      |
| `data-state`                | `"stacked"`           | Stack layout state.           | `"stacked"` |
| `data-axis`                 | `"block"`, `"inline"` | Logical axis styling hook.    | `"block"`   |
| `data-reversed`             | `"true"`, `"false"`   | Reverse-axis styling hook.    | `"false"`   |
| `data-vize-stack-direction` | `StackFlexDirection`  | Resolved flex direction hook. | `"column"`  |
| `data-vize-stack-gap`       | `string`              | Resolved gap hook.            | `"1rem"`    |
| `data-vize-stack-align`     | `StackAlign`          | Resolved alignment hook.      | `"stretch"` |
| `data-vize-stack-justify`   | `StackJustify`        | Resolved justification hook.  | `"start"`   |

## ARIA Attributes

Stack never sets `role`, `aria-hidden`, `aria-label`, `aria-labelledby`, or
`tabindex`. Consumers may pass semantic attributes to the host when the chosen
element requires them.

## CSS Custom Properties

| Property                  | Purpose                                   | Default     |
| ------------------------- | ----------------------------------------- | ----------- |
| `--vize-ui-stack-gap`     | Value read by the host `gap`.             | `"1rem"`    |
| `--vize-ui-stack-align`   | Value read by the host `align-items`.     | `"stretch"` |
| `--vize-ui-stack-justify` | Value read by the host `justify-content`. | `"start"`   |

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
