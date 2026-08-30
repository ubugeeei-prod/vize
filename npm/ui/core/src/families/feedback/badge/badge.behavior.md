# Badge behavior contract

Normative state x input -> outcome table for `badge.vue` (`@vizejs/ui/badge`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                         | Proven by                                                                |
| --- | ------------- | ----------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| B1  | default       | render / Tab      | renders inline `<span data-vize-ui="badge">`, `part="root"`, label/neutral hooks, and no focus  | `renders an inline neutral label badge by default`                       |
| B2  | status/count  | render            | renders the requested host while mirroring strict variant and tone tokens to data hooks         | `renders status and count variants without adding ARIA or focus policy`  |
| B3  | consumer ARIA | fallthrough attrs | preserves consumer-owned role and live-region attributes without deriving any default semantics | `keeps ARIA and live-region semantics consumer owned through attrs`      |
| B4  | any           | slot/expose       | passes variant and tone to the slot and exposes the rendered element with live prop updates     | `passes slot state and exposes live badge state`                         |
| B5  | SSR default   | isolated requests | renders byte-identical default badge markup without request-global state                        | `renders byte-identical label badge markup across isolated SSR requests` |
| B6  | SSR custom    | render            | renders custom host markup with strict hooks and no default role, tabindex, or `aria-hidden`    | `renders custom server markup without implicit accessibility attributes` |
| B7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                       | `scripts/check-renderers.ts`                                             |
| B8  | root/subpath  | consumer bundle   | root and subpath consumers retain only Badge, emit no CSS, and stay within gzip budget          | `scripts/check-tree-shaking.mjs`                                         |

## Props

| Prop      | Type                                                                    | Purpose                                                        | Default     |
| --------- | ----------------------------------------------------------------------- | -------------------------------------------------------------- | ----------- |
| `as`      | `PrimitiveAs`                                                           | Native element, custom element, or component rendered as host. | `"span"`    |
| `variant` | `"count" \| "label" \| "status"`                                        | Badge usage variant mirrored to `data-variant`.                | `"label"`   |
| `tone`    | `"accent" \| "danger" \| "info" \| "neutral" \| "success" \| "warning"` | Consumer styling tone mirrored to `data-tone`.                 | `"neutral"` |

## Slots

| Slot      | Props                                        | Purpose                                            | Default |
| --------- | -------------------------------------------- | -------------------------------------------------- | ------- |
| `default` | `{ variant: BadgeVariant; tone: BadgeTone }` | Render semantic badge text or custom chip content. | none    |

## Expose

| Name      | Type                             | Purpose                                      | Default     |
| --------- | -------------------------------- | -------------------------------------------- | ----------- |
| `element` | `BadgeElement \| null`           | Rendered host element or component instance. | `null`      |
| `variant` | `"count" \| "label" \| "status"` | Badge usage variant.                         | `"label"`   |
| `tone`    | `BadgeTone`                      | Consumer styling tone.                       | `"neutral"` |

## Data Attributes

| Attribute      | Values                                                                  | Purpose                     | Default     |
| -------------- | ----------------------------------------------------------------------- | --------------------------- | ----------- |
| `data-vize-ui` | `"badge"`                                                               | Stable family selector.     | always      |
| `data-variant` | `"count"`, `"label"`, `"status"`                                        | Badge usage styling hook.   | `"label"`   |
| `data-tone`    | `"accent"`, `"danger"`, `"info"`, `"neutral"`, `"success"`, `"warning"` | Consumer tone styling hook. | `"neutral"` |

## ARIA Attributes

Badge never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Text content remains exposed through native
inline semantics. Consumers that need a live unread count, decorative status
dot, or labelled host pass ordinary Vue fallthrough attributes themselves.

## CSS Custom Properties

Badge defines no CSS custom properties and ships no stylesheet. Consumers own
shape, color, density, truncation, and icon spacing through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
