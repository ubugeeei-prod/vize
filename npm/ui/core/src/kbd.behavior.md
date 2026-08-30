# Kbd behavior contract

Normative state x input -> outcome table for `kbd.vue` (`@vizejs/ui/kbd`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                          | Proven by                                                                      |
| --- | ------------- | ----------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| K1  | default       | render / Tab      | renders native `<kbd data-vize-ui="kbd">`, `part="root"`, default hooks, slot text, and no focus | `renders native keyboard input by default without styling or focus policy`     |
| K2  | custom hooks  | render            | renders the requested host while mirroring strict size, variant, and tone hooks                  | `mirrors shortcut presentation hooks on a custom host`                         |
| K3  | consumer ARIA | fallthrough attrs | preserves consumer-owned role, label, and focus attributes without deriving custom semantics     | `keeps custom semantics and focus policy consumer owned through attrs`         |
| K4  | any           | slot/expose       | passes size, variant, and tone to the slot and exposes the rendered element with live props      | `passes slot state and exposes live kbd state`                                 |
| K5  | SSR default   | isolated requests | renders byte-identical native kbd markup without request-global state                            | `renders byte-identical native kbd markup across isolated SSR requests`        |
| K6  | SSR custom    | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style   | `renders consumer-owned server semantics without implicit accessibility attrs` |
| K7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                        | `scripts/check-renderers.ts`                                                   |
| K8  | SSR/hydration | hydrate           | server kbd markup hydrates without warnings, node replacement, or accessibility drift            | `src/runtime-conformance.test.ts`                                              |
| K9  | root/subpath  | consumer bundle   | root and subpath consumers retain only Kbd, emit no CSS, and stay within gzip budget             | `scripts/check-tree-shaking.mjs`                                               |

## Props

| Prop      | Type                                                                     | Purpose                                                        | Default     |
| --------- | ------------------------------------------------------------------------ | -------------------------------------------------------------- | ----------- |
| `as`      | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host. | `"kbd"`     |
| `size`    | `"sm" \| "md" \| "lg"`                                                   | Consumer visual-size token mirrored to `data-size`.            | `"md"`      |
| `variant` | `"key" \| "shortcut" \| "sequence"`                                      | Keyboard presentation token mirrored to `data-variant`.        | `"key"`     |
| `tone`    | `"neutral" \| "muted" \| "accent" \| "success" \| "warning" \| "danger"` | Consumer color or semantic tone token mirrored to `data-tone`. | `"neutral"` |

## Slots

| Slot      | Props                                                   | Purpose                                               | Default |
| --------- | ------------------------------------------------------- | ----------------------------------------------------- | ------- |
| `default` | `{ size: KbdSize; variant: KbdVariant; tone: KbdTone }` | Render keyboard input, shortcut, or sequence content. | none    |

## Expose

| Name      | Type                 | Purpose                                      | Default     |
| --------- | -------------------- | -------------------------------------------- | ----------- |
| `element` | `KbdElement \| null` | Rendered host element or component instance. | `null`      |
| `size`    | `KbdSize`            | Current visual-size token.                   | `"md"`      |
| `variant` | `KbdVariant`         | Current presentation token.                  | `"key"`     |
| `tone`    | `KbdTone`            | Current tone token.                          | `"neutral"` |

## Data Attributes

| Attribute      | Values                                                                   | Purpose                    | Default     |
| -------------- | ------------------------------------------------------------------------ | -------------------------- | ----------- |
| `data-vize-ui` | `"kbd"`                                                                  | Stable family selector.    | always      |
| `data-size`    | `"sm"`, `"md"`, `"lg"`                                                   | Consumer visual-size hook. | `"md"`      |
| `data-variant` | `"key"`, `"shortcut"`, `"sequence"`                                      | Presentation hook.         | `"key"`     |
| `data-tone`    | `"neutral"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` | Consumer tone hook.        | `"neutral"` |

## ARIA Attributes

Kbd never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Native `<kbd>` remains exposed as keyboard
input content. Consumers that need glossary terms, hidden separators, or
focusable shortcut controls pass ordinary Vue fallthrough attributes themselves.

## CSS Custom Properties

Kbd defines no CSS custom properties and ships no stylesheet. Consumers own
keycap borders, backgrounds, spacing, typography, separators, and wrapping
through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
