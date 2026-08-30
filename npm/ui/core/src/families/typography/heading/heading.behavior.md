# Heading behavior contract

Normative state x input -> outcome table for `heading.vue` (`@vizejs/ui/heading`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                                  | Proven by                                                                     |
| --- | ------------- | ----------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| H1  | default       | render / Tab      | renders semantic `<h2 data-vize-ui="heading">`, `part="root"`, default hooks, slot text, and no focus    | `renders a semantic h2 by default without adding focus or styling`            |
| H2  | custom level  | render / update   | derives the native heading host from `level` and updates the native host plus `data-level` reactively    | `derives the native heading host from level when as is omitted`               |
| H3  | custom host   | fallthrough attrs | preserves consumer-owned role, `aria-level`, and focus attributes without deriving custom-host semantics | `keeps custom host semantics and focus policy consumer owned`                 |
| H4  | any           | slot/expose       | passes level, size, weight, tone, and truncate to the slot and exposes the rendered element live         | `passes slot state and exposes live heading state`                            |
| H5  | SSR default   | isolated requests | renders byte-identical native heading markup without request-global state                                | `renders byte-identical semantic heading markup across isolated SSR requests` |
| H6  | SSR custom    | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style           | `renders consumer-owned custom heading semantics without implicit aria`       |
| H7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                                | `scripts/check-renderers.ts`                                                  |
| H8  | SSR/hydration | hydrate           | server heading markup hydrates without warnings, node replacement, or accessibility drift                | `src/runtime-conformance.test.ts`                                             |
| H9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Heading, emit no CSS, and stay within gzip budget                 | `scripts/check-tree-shaking.mjs`                                              |

## Props

| Prop       | Type                                                                     | Purpose                                                                               | Default      |
| ---------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------- | ------------ |
| `as`       | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host.                        | `undefined`  |
| `level`    | `1 \| 2 \| 3 \| 4 \| 5 \| 6`                                             | Semantic heading level used for the default native host and mirrored to `data-level`. | `2`          |
| `size`     | `"xs" \| "sm" \| "md" \| "lg" \| "xl" \| "2xl"`                          | Consumer visual-size token mirrored to `data-size`.                                   | `"md"`       |
| `weight`   | `"regular" \| "medium" \| "semibold" \| "bold"`                          | Consumer font-weight token mirrored to `data-weight`.                                 | `"semibold"` |
| `tone`     | `"neutral" \| "muted" \| "accent" \| "success" \| "warning" \| "danger"` | Consumer color or semantic tone token mirrored to `data-tone`.                        | `"neutral"`  |
| `truncate` | `boolean`                                                                | Consumer truncation hook mirrored to `data-truncate`; no CSS is emitted.              | `false`      |

## Slots

| Slot      | Props                                                                                                     | Purpose                                          | Default |
| --------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ------- |
| `default` | `{ level: HeadingLevel; size: HeadingSize; weight: HeadingWeight; tone: HeadingTone; truncate: boolean }` | Render semantic heading content or custom nodes. | none    |

## Expose

| Name       | Type                     | Purpose                                      | Default      |
| ---------- | ------------------------ | -------------------------------------------- | ------------ |
| `element`  | `HeadingElement \| null` | Rendered host element or component instance. | `null`       |
| `level`    | `HeadingLevel`           | Current semantic heading level.              | `2`          |
| `size`     | `HeadingSize`            | Current visual-size token.                   | `"md"`       |
| `weight`   | `HeadingWeight`          | Current weight token.                        | `"semibold"` |
| `tone`     | `HeadingTone`            | Current tone token.                          | `"neutral"`  |
| `truncate` | `boolean`                | Current truncation hook state.               | `false`      |

## Data Attributes

| Attribute       | Values                                                                   | Purpose                      | Default      |
| --------------- | ------------------------------------------------------------------------ | ---------------------------- | ------------ |
| `data-vize-ui`  | `"heading"`                                                              | Stable family selector.      | always       |
| `data-level`    | `"1"`, `"2"`, `"3"`, `"4"`, `"5"`, `"6"`                                 | Semantic heading-level hook. | `"2"`        |
| `data-size`     | `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"`, `"2xl"`                          | Consumer visual-size hook.   | `"md"`       |
| `data-weight`   | `"regular"`, `"medium"`, `"semibold"`, `"bold"`                          | Consumer font-weight hook.   | `"semibold"` |
| `data-tone`     | `"neutral"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` | Consumer tone hook.          | `"neutral"`  |
| `data-truncate` | `"true"`, `"false"`                                                      | Consumer truncation hook.    | `"false"`    |

## ARIA Attributes

Heading never sets `role`, `tabindex`, `aria-level`, `aria-hidden`,
`aria-live`, `aria-label`, or `aria-labelledby` by default. Native heading
elements carry document structure. Consumers that override `as` to a non-heading
host pass the matching role, level, and focus attributes explicitly.

## CSS Custom Properties

Heading defines no CSS custom properties and ships no stylesheet. Consumers own
font size, line height, font weight, color, truncation, wrapping, and margin
through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
