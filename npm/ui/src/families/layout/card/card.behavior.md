# Card behavior contract

Normative state x input -> outcome table for `card.vue` (`@vizejs/ui/card`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State              | Input             | Outcome                                                                                              | Proven by                                                                   |
| --- | ------------------ | ----------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| C1  | default            | render / Tab      | renders `<section data-vize-ui="card">`, `part="root"`, card/comfortable/neutral hooks, and no focus | `renders a neutral section card by default without styling or focus policy` |
| C2  | variant matrix     | render            | renders supported variant, density, and tone tokens as stable data hooks on the requested host       | `mirrors strict surface tokens without adding semantics`                    |
| C3  | consumer semantics | fallthrough attrs | preserves consumer-owned role, label, tabindex, and ARIA attributes without deriving defaults        | `keeps semantics and focus policy consumer owned through attrs`             |
| C4  | any                | slot/expose       | passes variant, density, and tone to the slot and exposes the rendered element with live updates     | `passes slot state and exposes live card state`                             |
| C5  | SSR default        | isolated requests | renders byte-identical default card markup without request-global state                              | `renders byte-identical default card markup across isolated SSR requests`   |
| C6  | SSR custom         | render            | renders custom host markup with strict hooks and only consumer-provided accessibility attributes     | `renders custom server markup without implicit accessibility attributes`    |
| C7  | SSR/hydration      | runtime fixture   | server markup hydrates without warnings or root node replacement                                     | `runtime-conformance.test.ts`                                               |
| C8  | DOM/SSR/Vapor      | compile           | authored SFC compiles in every renderer lane without warnings or fallback                            | `scripts/check-renderers.ts`                                                |
| C9  | root/subpath       | consumer bundle   | root and subpath consumers retain only Card, emit no CSS, and stay within gzip budget                | `scripts/check-tree-shaking.mjs`                                            |

## Props

| Prop      | Type                                                                    | Purpose                                                        | Default         |
| --------- | ----------------------------------------------------------------------- | -------------------------------------------------------------- | --------------- |
| `as`      | `PrimitiveAs`                                                           | Native element, custom element, or component rendered as host. | `"section"`     |
| `variant` | `"card" \| "panel" \| "surface"`                                        | Surface usage variant mirrored to `data-variant`.              | `"card"`        |
| `density` | `"compact" \| "comfortable" \| "spacious"`                              | Spacing density token mirrored to `data-density`.              | `"comfortable"` |
| `tone`    | `"neutral" \| "accent" \| "info" \| "success" \| "warning" \| "danger"` | Consumer styling tone mirrored to `data-tone`.                 | `"neutral"`     |

## Slots

| Slot      | Props                                                            | Purpose                                | Default |
| --------- | ---------------------------------------------------------------- | -------------------------------------- | ------- |
| `default` | `{ variant: CardVariant; density: CardDensity; tone: CardTone }` | Render consumer-owned surface content. | none    |

## Expose

| Name      | Type                  | Purpose                                      | Default         |
| --------- | --------------------- | -------------------------------------------- | --------------- |
| `element` | `CardElement \| null` | Rendered host element or component instance. | `null`          |
| `variant` | `CardVariant`         | Surface usage variant.                       | `"card"`        |
| `density` | `CardDensity`         | Surface density token.                       | `"comfortable"` |
| `tone`    | `CardTone`            | Consumer styling tone.                       | `"neutral"`     |

## Data Attributes

| Attribute      | Values                                                                  | Purpose                     | Default         |
| -------------- | ----------------------------------------------------------------------- | --------------------------- | --------------- |
| `data-vize-ui` | `"card"`                                                                | Stable family selector.     | always          |
| `data-variant` | `"card"`, `"panel"`, `"surface"`                                        | Surface usage styling hook. | `"card"`        |
| `data-density` | `"compact"`, `"comfortable"`, `"spacious"`                              | Density styling hook.       | `"comfortable"` |
| `data-tone`    | `"neutral"`, `"accent"`, `"info"`, `"success"`, `"warning"`, `"danger"` | Consumer tone styling hook. | `"neutral"`     |

## ARIA Attributes

Card never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Consumers choose whether a surface is a
region, article, list item, form group, decorative wrapper, or focusable panel
by passing ordinary Vue fallthrough attributes.

## CSS Custom Properties

Card defines no CSS custom properties and ships no stylesheet. Consumers own
shape, border, shadow, spacing, color, elevation, and responsive treatment
through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
