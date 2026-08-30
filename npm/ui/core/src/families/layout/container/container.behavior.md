# Container behavior contract

Normative state x input -> outcome table for `container.vue` (`@vizejs/ui/container`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                            | Proven by                                                                       |
| --- | ------------- | ----------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| C1  | default       | resolve           | maps the `md` preset to `64rem`, normalizes padding to `0`, and centers with logical auto margins  | `resolves a centered default container with no authored CSS classes`            |
| C2  | overrides     | resolve           | numeric `maxInlineSize` and `paddingInline` values resolve to px strings without centering margins | `resolves preset and numeric overrides into native logical CSS values`          |
| C3  | default       | render            | renders a non-focusable `<div>` container with part, data hooks, logical style hooks, and slots    | `renders a non-focusable container by default while preserving child semantics` |
| C4  | custom host   | render            | forwards host attributes and renders uncentered max-inline-size and padding-inline hooks           | `renders an uncentered custom semantic host with forwarded attributes`          |
| C5  | any           | slot/expose       | exposes `element`, `size`, `maxInlineSize`, `paddingInline`, `centered`, and the resolved style    | `passes slot state and exposes live resolved logical sizing state`              |
| C6  | SSR           | isolated requests | renders byte-identical logical container markup without request-global state                       | `renders byte-identical container markup across isolated SSR requests`          |
| C7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                          | `scripts/check-renderers.ts`                                                    |
| C8  | root/subpath  | consumer bundle   | root and subpath consumers retain only Container, emit no CSS, and stay within gzip budget         | `scripts/check-tree-shaking.mjs`                                                |

## Props

| Prop            | Type                                             | Purpose                                                       | Default     |
| --------------- | ------------------------------------------------ | ------------------------------------------------------------- | ----------- |
| `as`            | `PrimitiveAs`                                    | Native element, custom element, or component host.            | `"div"`     |
| `size`          | `"xs" \| "sm" \| "md" \| "lg" \| "xl" \| "full"` | Named max-inline-size preset.                                 | `"md"`      |
| `maxInlineSize` | `string \| number`                               | Native CSS `max-inline-size` override; numbers resolve to px. | `undefined` |
| `paddingInline` | `string \| number`                               | Native CSS `padding-inline` value; numbers resolve to px.     | `0`         |
| `centered`      | `boolean`                                        | Center the host with logical inline auto margins.             | `true`      |

## Size Presets

| Size   | Max inline size |
| ------ | --------------- |
| `xs`   | `36rem`         |
| `sm`   | `48rem`         |
| `md`   | `64rem`         |
| `lg`   | `80rem`         |
| `xl`   | `96rem`         |
| `full` | `none`          |

## Slots

| Slot      | Props                | Purpose                            | Default |
| --------- | -------------------- | ---------------------------------- | ------- |
| `default` | `ContainerSlotState` | Renders direct container children. | empty   |

## Expose

| Name            | Type                       | Purpose                                      | Default |
| --------------- | -------------------------- | -------------------------------------------- | ------- |
| `element`       | `ContainerElement \| null` | Rendered host element or component instance. | `null`  |
| `size`          | `ContainerSize`            | Resolved named size preset.                  | `"md"`  |
| `maxInlineSize` | `ContainerResolvedLength`  | Resolved CSS max inline size value.          | `64rem` |
| `paddingInline` | `ContainerResolvedLength`  | Resolved CSS inline padding value.           | `0`     |
| `centered`      | `boolean`                  | Whether logical auto margins are applied.    | `true`  |
| `style`         | `ContainerStyle`           | Inline logical style hooks applied to host.  | object  |

## Data Attributes

| Attribute       | Values              | Purpose                 | Default  |
| --------------- | ------------------- | ----------------------- | -------- |
| `data-vize-ui`  | `"container"`       | Stable family selector. | always   |
| `data-size`     | `ContainerSize`     | Resolved size hook.     | `"md"`   |
| `data-centered` | `"true"`, `"false"` | Centering hook.         | `"true"` |

## ARIA Attributes

Container never sets `role`, `aria-hidden`, `aria-label`, `aria-labelledby`, or
`tabindex`. Consumers may pass semantic attributes to the host when the chosen
element requires them.

## CSS Custom Properties

| Property                              | Purpose                                   | Default |
| ------------------------------------- | ----------------------------------------- | ------- |
| `--vize-ui-container-max-inline-size` | Value read by the host `max-inline-size`. | `64rem` |
| `--vize-ui-container-padding-inline`  | Value read by the host `padding-inline`.  | `0`     |

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
