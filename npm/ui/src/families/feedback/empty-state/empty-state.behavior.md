# EmptyState behavior contract

Normative state x input -> outcome table for `empty-state.vue`
(`@vizejs/ui/empty-state`). Every row is proven by the named mounted-DOM or
SSR test. A row without a passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                                   | Proven by                                                                        |
| --- | ------------- | ----------------- | --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| E1  | default       | render / Tab      | renders `<section data-vize-ui="empty-state">`, `part="root"`, neutral/comfortable/block hooks, no focus  | `renders a neutral section empty state by default`                               |
| E2  | custom hooks  | render            | renders the requested host while mirroring strict tone, density, orientation, and empty-state tokens      | `renders custom hooks without adding accessibility or focus policy`              |
| E3  | consumer ARIA | fallthrough attrs | preserves consumer-owned role, labels, live-region policy, and focus attrs without deriving defaults      | `keeps labels, roles, live-region policy, and focus attrs consumer owned`        |
| E4  | any           | slot/expose       | passes tone, density, orientation, and state to the slot and exposes the rendered element live            | `passes slot state and exposes live empty-state hooks`                           |
| E5  | SSR default   | isolated requests | renders byte-identical default empty-state markup without request-global state                            | `renders byte-identical default empty-state markup across isolated SSR requests` |
| E6  | SSR custom    | render            | renders custom host markup with strict hooks and no default role, tabindex, `aria-hidden`, or live region | `renders custom server markup without implicit accessibility attributes`         |
| E7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                                 | `scripts/check-renderers.ts`                                                     |
| E8  | root/subpath  | consumer bundle   | root and subpath consumers retain only EmptyState, emit no CSS, and stay within gzip budget               | `scripts/check-tree-shaking.mjs`                                                 |

## Props

| Prop          | Type                                                        | Purpose                                                        | Default         |
| ------------- | ----------------------------------------------------------- | -------------------------------------------------------------- | --------------- |
| `as`          | `PrimitiveAs`                                               | Native element, custom element, or component rendered as host. | `"section"`     |
| `tone`        | `"neutral" \| "info" \| "success" \| "warning" \| "danger"` | Consumer styling tone mirrored to `data-tone`.                 | `"neutral"`     |
| `density`     | `"compact" \| "comfortable"`                                | Consumer spacing density mirrored to `data-density`.           | `"comfortable"` |
| `orientation` | `"block" \| "inline"`                                       | Consumer layout orientation mirrored to `data-orientation`.    | `"block"`       |

## Slots

| Slot      | Props                                                                                                      | Purpose                                 | Default |
| --------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------------- | ------- |
| `default` | `{ tone: EmptyStateTone; density: EmptyStateDensity; orientation: EmptyStateOrientation; state: "empty" }` | Render empty-state content and actions. | none    |

## Expose

| Name          | Type                        | Purpose                                      | Default         |
| ------------- | --------------------------- | -------------------------------------------- | --------------- |
| `element`     | `EmptyStateElement \| null` | Rendered host element or component instance. | `null`          |
| `tone`        | `EmptyStateTone`            | Consumer styling tone.                       | `"neutral"`     |
| `density`     | `EmptyStateDensity`         | Consumer spacing density.                    | `"comfortable"` |
| `orientation` | `EmptyStateOrientation`     | Consumer layout orientation.                 | `"block"`       |
| `state`       | `"empty"`                   | Stable empty-state token.                    | `"empty"`       |

## Data Attributes

| Attribute          | Values                                                      | Purpose                        | Default         |
| ------------------ | ----------------------------------------------------------- | ------------------------------ | --------------- |
| `data-vize-ui`     | `"empty-state"`                                             | Stable family selector.        | always          |
| `data-state`       | `"empty"`                                                   | Stable state styling hook.     | `"empty"`       |
| `data-tone`        | `"neutral"`, `"info"`, `"success"`, `"warning"`, `"danger"` | Consumer tone styling hook.    | `"neutral"`     |
| `data-density`     | `"compact"`, `"comfortable"`                                | Consumer density styling hook. | `"comfortable"` |
| `data-orientation` | `"block"`, `"inline"`                                       | Consumer layout styling hook.  | `"block"`       |

## ARIA Attributes

EmptyState never sets `role`, `tabindex`, `aria-hidden`, `aria-live`,
`aria-label`, `aria-labelledby`, or `aria-describedby` by default. Consumers
that need landmarks, status announcements, decorative treatment, retry focus,
or labels pass ordinary Vue fallthrough attributes themselves.

## CSS Custom Properties

EmptyState defines no CSS custom properties and ships no stylesheet. Consumers
own spacing, icon layout, call-to-action alignment, color, and responsive
composition through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
