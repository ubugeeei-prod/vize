# StatusLight behavior contract

Normative state x input -> outcome table for `status-light.vue`
(`@vizejs/ui/status-light` sidecar). Every row is proven by the named
mounted-DOM, SSR/hydration, renderer, or compile-only type test. A row without
a passing test is a contract violation.

| #   | State          | Input             | Outcome                                                                                 | Proven by                                                          |
| --- | -------------- | ----------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| L1  | default        | render            | renders a headless decorative `<span>` with stable state, tone, size, part, and no tab  | `renders a decorative neutral unknown light by default`            |
| L2  | labelled image | label/description | exposes `role="img"` plus consumer-owned name and description attributes                | `renders labelled image semantics with description support`        |
| L3  | status role    | labelledby        | exposes a polite status role and optional atomic policy without adding focus behavior   | `supports status announcements and labelledby names`               |
| L4  | decorative     | `ariaHidden`      | suppresses role, name, description, live-region attributes, and status queries          | `lets ariaHidden override labelled status semantics`               |
| L5  | reactive       | prop update       | updates data attributes, slot state, and exposed state without replacing the host       | `passes slot state and exposes live status-light state`            |
| L6  | SSR image      | isolated requests | renders byte-identical labelled image markup with no class, style, tab, or handler leak | `renders byte-identical image markup across isolated SSR requests` |
| L7  | hydration      | server markup     | hydrates the server-rendered host in place with no diagnostics                          | `hydrates labelled markup without replacing the status-light root` |
| L8  | SSR status     | render            | renders status-role server markup with polite live-region and explicit atomic state     | `renders server status markup with consumer-owned labels`          |
| L9  | public types   | invalid contract  | TypeScript rejects unsupported state, tone, size, role, and malformed slot state tokens | `src/families/feedback/status-light/status-light.types.test-d.ts`  |

## Props

| Prop              | Type                                                                    | Purpose                                                                   | Default     |
| ----------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------- |
| `as`              | `PrimitiveAs`                                                           | Native element, custom element, or component rendered as host.            | `"span"`    |
| `state`           | `"away" \| "busy" \| "offline" \| "online" \| "unknown"`                | Presence or health state mirrored to `data-state`.                        | `"unknown"` |
| `tone`            | `"accent" \| "danger" \| "info" \| "neutral" \| "success" \| "warning"` | Consumer styling tone mirrored to `data-tone`.                            | `"neutral"` |
| `size`            | `"sm" \| "md" \| "lg"`                                                  | Consumer size token mirrored to `data-size`.                              | `"md"`      |
| `role`            | `"img" \| "status"`                                                     | Accessibility role used unless decorative.                                | `"img"`     |
| `atomic`          | `boolean`                                                               | Whether `role="status"` announcements should be atomic.                   | `true`      |
| `ariaHidden`      | `boolean`                                                               | Forces decorative semantics when true; false allows slot-authored naming. | `undefined` |
| `ariaLabel`       | `string`                                                                | Accessible name when no visible label or `aria-labelledby` supplies one.  | `undefined` |
| `ariaLabelledby`  | `string`                                                                | Space-separated ids that label the status light.                          | `undefined` |
| `ariaDescribedby` | `string`                                                                | Space-separated ids that describe the status light.                       | `undefined` |

## Slots

| Slot      | Props                  | Purpose                                     | Default |
| --------- | ---------------------- | ------------------------------------------- | ------- |
| `default` | `StatusLightSlotState` | Render the consumer-owned visual indicator. | none    |

## Expose

| Name         | Type                         | Purpose                             | Default        |
| ------------ | ---------------------------- | ----------------------------------- | -------------- |
| `element`    | `StatusLightElement \| null` | Rendered host element or component. | `null`         |
| `state`      | `StatusLightState`           | Presence or health state token.     | `"unknown"`    |
| `tone`       | `StatusLightTone`            | Consumer styling tone token.        | `"neutral"`    |
| `size`       | `StatusLightSize`            | Consumer size token.                | `"md"`         |
| `ariaState`  | `StatusLightAriaState`       | Resolved accessibility policy.      | `"decorative"` |
| `decorative` | `boolean`                    | Whether the host is hidden from AT. | `true`         |

## Data Attributes

| Attribute         | Values                                                                  | Purpose                    | Default        |
| ----------------- | ----------------------------------------------------------------------- | -------------------------- | -------------- |
| `data-vize-ui`    | `"status-light"`                                                        | Stable family selector.    | always         |
| `data-state`      | `"away"`, `"busy"`, `"offline"`, `"online"`, `"unknown"`                | Presence or health state.  | `"unknown"`    |
| `data-tone`       | `"accent"`, `"danger"`, `"info"`, `"neutral"`, `"success"`, `"warning"` | Consumer styling tone.     | `"neutral"`    |
| `data-size`       | `"sm"`, `"md"`, `"lg"`                                                  | Consumer size token.       | `"md"`         |
| `data-aria-state` | `"decorative"`, `"img"`, `"status"`                                     | Accessibility policy hook. | `"decorative"` |
| `data-decorative` | `"true"`, `"false"`                                                     | Decorative-state hook.     | `"true"`       |

## ARIA Attributes

| Attribute          | Values                | Purpose                                            | Default     |
| ------------------ | --------------------- | -------------------------------------------------- | ----------- |
| `role`             | `"img"` or `"status"` | Names the light when it is not decorative.         | `undefined` |
| `aria-hidden`      | `"true"`              | Hides decorative lights from assistive technology. | `"true"`    |
| `aria-label`       | `string`              | Optional accessible name.                          | `undefined` |
| `aria-labelledby`  | `string`              | Optional external accessible name.                 | `undefined` |
| `aria-describedby` | `string`              | Optional external accessible description.          | `undefined` |
| `aria-live`        | `"polite"`            | Status live-region politeness for `role="status"`. | `undefined` |
| `aria-atomic`      | `"true"` or `"false"` | Status live-region atomicity for `role="status"`.  | `undefined` |

## Parts

| Part   | Element | Purpose                              |
| ------ | ------- | ------------------------------------ |
| `root` | host    | Style the rendered StatusLight host. |

## Styling Contract

StatusLight is headless: it emits no visual CSS, animation, SVG, color preset,
or size preset. Consumers provide all pixels through the default slot or CSS
using the root part and stable data attributes.
