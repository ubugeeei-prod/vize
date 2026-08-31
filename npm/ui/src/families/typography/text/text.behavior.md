# Text behavior contract

Normative state x input -> outcome table for `text.vue` (`@vizejs/ui/text`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                              | Proven by                                                                           |
| --- | ------------- | ----------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| T1  | default       | render / Tab      | renders inline `<span data-vize-ui="text">`, `part="root"`, default hooks, slot text, and no focus   | `renders neutral body text by default without adding semantics or styling`          |
| T2  | custom tokens | render            | renders the requested host while mirroring strict size, weight, tone, and truncation hooks           | `mirrors typography tokens and truncation intent on a custom host`                  |
| T3  | consumer ARIA | fallthrough attrs | preserves consumer-owned role, live-region, and focus attributes without deriving default semantics  | `keeps ARIA and focus policy consumer owned through attrs`                          |
| T4  | any           | slot/expose       | passes size, weight, tone, and truncate to the slot and exposes the rendered element with live props | `passes slot state and exposes live text state`                                     |
| T5  | SSR default   | isolated requests | renders byte-identical default text markup without request-global state                              | `renders byte-identical neutral text markup across isolated SSR requests`           |
| T6  | SSR custom    | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style       | `renders consumer-owned server semantics without implicit accessibility attributes` |
| T7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                            | `scripts/check-renderers.ts`                                                        |
| T8  | SSR/hydration | hydrate           | server text markup hydrates without warnings, node replacement, or accessibility drift               | `src/conformance/runtime-conformance.test.ts`                                       |
| T9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Text, emit no CSS, and stay within gzip budget                | `scripts/check-tree-shaking.mjs`                                                    |

## Props

| Prop       | Type                                                                     | Purpose                                                                  | Default     |
| ---------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------ | ----------- |
| `as`       | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host.           | `"span"`    |
| `size`     | `"xs" \| "sm" \| "md" \| "lg" \| "xl"`                                   | Consumer text-size token mirrored to `data-size`.                        | `"md"`      |
| `weight`   | `"regular" \| "medium" \| "semibold" \| "bold"`                          | Consumer font-weight token mirrored to `data-weight`.                    | `"regular"` |
| `tone`     | `"neutral" \| "muted" \| "accent" \| "success" \| "warning" \| "danger"` | Consumer color or semantic tone token mirrored to `data-tone`.           | `"neutral"` |
| `truncate` | `boolean`                                                                | Consumer truncation hook mirrored to `data-truncate`; no CSS is emitted. | `false`     |

## Slots

| Slot      | Props                                                                       | Purpose                                      | Default |
| --------- | --------------------------------------------------------------------------- | -------------------------------------------- | ------- |
| `default` | `{ size: TextSize; weight: TextWeight; tone: TextTone; truncate: boolean }` | Render semantic text or custom inline nodes. | none    |

## Expose

| Name       | Type                  | Purpose                                      | Default     |
| ---------- | --------------------- | -------------------------------------------- | ----------- |
| `element`  | `TextElement \| null` | Rendered host element or component instance. | `null`      |
| `size`     | `TextSize`            | Current size token.                          | `"md"`      |
| `weight`   | `TextWeight`          | Current weight token.                        | `"regular"` |
| `tone`     | `TextTone`            | Current tone token.                          | `"neutral"` |
| `truncate` | `boolean`             | Current truncation hook state.               | `false`     |

## Data Attributes

| Attribute       | Values                                                                   | Purpose                    | Default     |
| --------------- | ------------------------------------------------------------------------ | -------------------------- | ----------- |
| `data-vize-ui`  | `"text"`                                                                 | Stable family selector.    | always      |
| `data-size`     | `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"`                                   | Consumer text-size hook.   | `"md"`      |
| `data-weight`   | `"regular"`, `"medium"`, `"semibold"`, `"bold"`                          | Consumer font-weight hook. | `"regular"` |
| `data-tone`     | `"neutral"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` | Consumer tone hook.        | `"neutral"` |
| `data-truncate` | `"true"`, `"false"`                                                      | Consumer truncation hook.  | `"false"`   |

## ARIA Attributes

Text never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Text content remains exposed through native
inline semantics. Consumers that need status text, decorative copy, or focusable
hosts pass ordinary Vue fallthrough attributes themselves.

## CSS Custom Properties

Text defines no CSS custom properties and ships no stylesheet. Consumers own
font size, line height, font weight, color, truncation, wrapping, and inline
spacing through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
