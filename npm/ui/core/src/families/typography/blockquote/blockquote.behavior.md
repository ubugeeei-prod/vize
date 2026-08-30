# Blockquote behavior contract

Normative state x input -> outcome table for `blockquote.vue` (`@vizejs/ui/blockquote`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State              | Input             | Outcome                                                                                                        | Proven by                                                                      |
| --- | ------------------ | ----------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| B1  | default            | render / Tab      | renders native `<blockquote data-vize-ui="blockquote">`, `part="root"`, default hooks, slot text, and no focus | `renders native blockquote by default without adding semantics or styling`     |
| B2  | quote hooks        | render            | renders strict size, tone, and native `cite` hooks on the root                                                 | `mirrors quote hooks and native citation on the root`                          |
| B3  | consumer semantics | fallthrough attrs | preserves consumer-owned role, label, and focus attributes without deriving custom semantics                   | `keeps custom semantics and focus policy consumer owned through attrs`         |
| B4  | any                | slot/expose       | passes size, tone, and cite to the slot and exposes the rendered element with live props                       | `passes slot state and exposes live blockquote state`                          |
| B5  | SSR default        | isolated requests | renders byte-identical native blockquote markup without request-global state                                   | `renders byte-identical native blockquote markup across isolated SSR requests` |
| B6  | SSR custom         | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style                 | `renders consumer-owned server semantics without implicit accessibility attrs` |
| B7  | DOM/SSR/Vapor      | compile           | authored SFC compiles in every renderer lane without warnings or fallback                                      | `scripts/check-renderers.ts`                                                   |
| B8  | SSR/hydration      | hydrate           | server blockquote markup hydrates without warnings, node replacement, or accessibility drift                   | `src/runtime-conformance.test.ts`                                              |
| B9  | root/subpath       | consumer bundle   | root and subpath consumers retain only Blockquote, emit no CSS, and stay within gzip budget                    | `scripts/check-tree-shaking.mjs`                                               |

## Props

| Prop   | Type                                                                     | Purpose                                                        | Default        |
| ------ | ------------------------------------------------------------------------ | -------------------------------------------------------------- | -------------- |
| `as`   | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host. | `"blockquote"` |
| `size` | `"sm" \| "md" \| "lg"`                                                   | Consumer quote-size token mirrored to `data-size`.             | `"md"`         |
| `tone` | `"neutral" \| "muted" \| "accent" \| "success" \| "warning" \| "danger"` | Consumer color or semantic tone token mirrored to `data-tone`. | `"neutral"`    |
| `cite` | `string \| undefined`                                                    | Native citation URL mirrored to the root `cite` attribute.     | `undefined`    |

## Slots

| Slot      | Props                                                                       | Purpose                                  | Default |
| --------- | --------------------------------------------------------------------------- | ---------------------------------------- | ------- |
| `default` | `{ size: BlockquoteSize; tone: BlockquoteTone; cite: string \| undefined }` | Render quoted content or attribution UI. | none    |

## Expose

| Name      | Type                        | Purpose                                      | Default     |
| --------- | --------------------------- | -------------------------------------------- | ----------- |
| `element` | `BlockquoteElement \| null` | Rendered host element or component instance. | `null`      |
| `size`    | `BlockquoteSize`            | Current visual-size token.                   | `"md"`      |
| `tone`    | `BlockquoteTone`            | Current tone token.                          | `"neutral"` |
| `cite`    | `string \| undefined`       | Current native citation URL attribute.       | `undefined` |

## Data Attributes

| Attribute      | Values                                                                   | Purpose                    | Default     |
| -------------- | ------------------------------------------------------------------------ | -------------------------- | ----------- |
| `data-vize-ui` | `"blockquote"`                                                           | Stable family selector.    | always      |
| `data-size`    | `"sm"`, `"md"`, `"lg"`                                                   | Consumer visual-size hook. | `"md"`      |
| `data-tone`    | `"neutral"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` | Consumer tone hook.        | `"neutral"` |

## Native Attributes

When `cite` is provided, Blockquote mirrors it to the rendered root's native
`cite` attribute. It does not render attribution text, `footer`, or `figcaption`
nodes; consumers own visible citation structure through the default slot.

## ARIA Attributes

Blockquote never sets `role`, `tabindex`, `aria-hidden`, `aria-live`,
`aria-label`, or `aria-labelledby` by default. Native `<blockquote>` remains
exposed as quoted flow content. Consumers that need a labelled note, figure,
region, focusable pull quote, or hidden decorative quote pass ordinary Vue
fallthrough attributes themselves.

## CSS Custom Properties

Blockquote defines no CSS custom properties and ships no stylesheet. Consumers
own quote marks, borders, indentation, spacing, typography, color, citation
layout, and responsive treatment through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
