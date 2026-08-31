# Code behavior contract

Normative state x input -> outcome table for `code.vue` (`@vizejs/ui/code`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                            | Proven by                                                                      |
| --- | ------------- | ----------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| C1  | default       | render / Tab      | renders native `<code data-vize-ui="code">`, `part="root"`, default hooks, slot text, and no focus | `renders native code by default without adding semantics or styling`           |
| C2  | custom hooks  | render            | renders the requested host while mirroring strict size, variant, and tone hooks                    | `mirrors code presentation hooks on a custom host`                             |
| C3  | consumer ARIA | fallthrough attrs | preserves consumer-owned role, label, and focus attributes without deriving custom semantics       | `keeps custom semantics and focus policy consumer owned through attrs`         |
| C4  | any           | slot/expose       | passes size, variant, and tone to the slot and exposes the rendered element with live props        | `passes slot state and exposes live code state`                                |
| C5  | SSR default   | isolated requests | renders byte-identical native code markup without request-global state                             | `renders byte-identical native code markup across isolated SSR requests`       |
| C6  | SSR custom    | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style     | `renders consumer-owned server semantics without implicit accessibility attrs` |
| C7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                          | `scripts/check-renderers.ts`                                                   |
| C8  | SSR/hydration | hydrate           | server code markup hydrates without warnings, node replacement, or accessibility drift             | `src/conformance/runtime-conformance.test.ts`                                  |
| C9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Code, emit no CSS, and stay within gzip budget              | `scripts/check-tree-shaking.mjs`                                               |

## Props

| Prop      | Type                                                                     | Purpose                                                        | Default     |
| --------- | ------------------------------------------------------------------------ | -------------------------------------------------------------- | ----------- |
| `as`      | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host. | `"code"`    |
| `size`    | `"sm" \| "md" \| "lg"`                                                   | Consumer code-size token mirrored to `data-size`.              | `"md"`      |
| `variant` | `"inline" \| "block" \| "snippet"`                                       | Code presentation token mirrored to `data-variant`.            | `"inline"`  |
| `tone`    | `"neutral" \| "muted" \| "accent" \| "success" \| "warning" \| "danger"` | Consumer color or semantic tone token mirrored to `data-tone`. | `"neutral"` |

## Slots

| Slot      | Props                                                      | Purpose                                      | Default |
| --------- | ---------------------------------------------------------- | -------------------------------------------- | ------- |
| `default` | `{ size: CodeSize; variant: CodeVariant; tone: CodeTone }` | Render inline code, block code, or snippets. | none    |

## Expose

| Name      | Type                  | Purpose                                      | Default     |
| --------- | --------------------- | -------------------------------------------- | ----------- |
| `element` | `CodeElement \| null` | Rendered host element or component instance. | `null`      |
| `size`    | `CodeSize`            | Current visual-size token.                   | `"md"`      |
| `variant` | `CodeVariant`         | Current presentation token.                  | `"inline"`  |
| `tone`    | `CodeTone`            | Current tone token.                          | `"neutral"` |

## Data Attributes

| Attribute      | Values                                                                   | Purpose                    | Default     |
| -------------- | ------------------------------------------------------------------------ | -------------------------- | ----------- |
| `data-vize-ui` | `"code"`                                                                 | Stable family selector.    | always      |
| `data-size`    | `"sm"`, `"md"`, `"lg"`                                                   | Consumer visual-size hook. | `"md"`      |
| `data-variant` | `"inline"`, `"block"`, `"snippet"`                                       | Presentation hook.         | `"inline"`  |
| `data-tone`    | `"neutral"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` | Consumer tone hook.        | `"neutral"` |

## ARIA Attributes

Code never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Native `<code>` remains exposed as code
content. Consumers that need a labelled region, term, focusable command
example, or hidden decorative code pass ordinary Vue fallthrough attributes
themselves.

## CSS Custom Properties

Code defines no CSS custom properties and ships no stylesheet. Consumers own
monospace font selection, syntax color, wrapping, white-space, indentation,
block layout, and inline rhythm through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
