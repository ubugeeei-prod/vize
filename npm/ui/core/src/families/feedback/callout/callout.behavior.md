# Callout behavior contract

Normative state x input -> outcome table for `callout.vue`
(`@vizejs/ui/callout`). Every row is proven by the named mounted-DOM,
SSR/hydration, renderer, or compile-only type test. A row without a passing
test is a contract violation.

| #   | State         | Input             | Outcome                                                                                                     | Proven by                                                                        |
| --- | ------------- | ----------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| C1  | default note  | render / Tab      | renders a headless `<section role="note">` with root/content parts, strict hooks, and no root focus         | `renders a labelled static note with structured parts by default`                |
| C2  | titled note   | title/description | generates SSR-stable title and description ids and wires them through ARIA unless consumers override        | `renders a labelled static note with structured parts by default`                |
| C3  | status        | render / actions  | exposes polite live-region semantics, optional atomic policy, trimmed direct naming, and tabbable actions   | `supports polite status semantics with direct naming and interactive actions`    |
| C4  | alert         | labelledby        | exposes assertive alert semantics while preserving normalized consumer-owned label and description ids      | `supports assertive alerts with consumer-owned title and description ids`        |
| C5  | closed/hidden | `open`/hidden     | remains mounted, mirrors `data-state="closed"`, suppresses ARIA when closed or decorative, and avoids focus | `closed and decorative callouts stay mounted without announcing`                 |
| C6  | reactive      | prop update       | updates data attributes, slot state, and exposed hooks without replacing the host                           | `passes slot state and exposes live Callout hooks`                               |
| C7  | SSR note      | isolated requests | renders byte-identical structured note markup with no class, style, tab, handler, or live leak              | `renders byte-identical labelled note markup across isolated SSR requests`       |
| C8  | hydration     | server markup     | hydrates generated title/description references in place with no diagnostics                                | `hydrates generated title and description references without replacing the root` |
| C9  | SSR alert     | render            | renders assertive server markup only when alert semantics are requested                                     | `renders assertive server markup when alert semantics are requested`             |
| C10 | public types  | invalid contract  | TypeScript rejects unsupported roles, tones, densities, booleans, and malformed slot state                  | `src/families/feedback/callout/callout.types.test-d.ts`                          |

## Props

| Prop              | Type                                                                    | Purpose                                                                          | Default         |
| ----------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------- | --------------- |
| `as`              | `PrimitiveAs`                                                           | Native element, custom element, or component rendered as host.                   | `"section"`     |
| `id`              | `string`                                                                | Consumer-owned root id for anchors or application state.                         | `undefined`     |
| `role`            | `"note" \| "status" \| "alert"`                                         | Static note, polite status, or assertive alert semantics.                        | `"note"`        |
| `open`            | `boolean`                                                               | Whether the Callout is visible.                                                  | `true`          |
| `atomic`          | `boolean`                                                               | Whether live-region updates should be atomic for `status` and `alert`.           | `true`          |
| `tone`            | `"accent" \| "danger" \| "info" \| "neutral" \| "success" \| "warning"` | Consumer styling tone mirrored to `data-tone`.                                   | `"neutral"`     |
| `density`         | `"compact" \| "comfortable"`                                            | Consumer density mirrored to `data-density`.                                     | `"comfortable"` |
| `iconAriaHidden`  | `boolean`                                                               | Whether the icon wrapper is decorative.                                          | `true`          |
| `ariaHidden`      | `boolean`                                                               | Forces decorative semantics for the entire Callout.                              | `undefined`     |
| `ariaLabel`       | `string`                                                                | Trimmed accessible name when no visible title or `aria-labelledby` supplies one. | `undefined`     |
| `ariaLabelledby`  | `string`                                                                | Space-separated ids that label the Callout after whitespace normalization.       | title slot id   |
| `ariaDescribedby` | `string`                                                                | Space-separated ids that describe the Callout after whitespace normalization.    | description id  |
| `titleId`         | `string`                                                                | Consumer-owned id for the title slot wrapper.                                    | generated       |
| `descriptionId`   | `string`                                                                | Consumer-owned id for the description slot wrapper.                              | generated       |

## Slots

| Slot          | Props              | Purpose                                       | Default |
| ------------- | ------------------ | --------------------------------------------- | ------- |
| `default`     | `CalloutSlotState` | Render the main message body.                 | none    |
| `icon`        | `CalloutSlotState` | Render an optional consumer-owned icon.       | none    |
| `title`       | `CalloutSlotState` | Render the accessible title.                  | none    |
| `description` | `CalloutSlotState` | Render the accessible description.            | none    |
| `actions`     | `CalloutSlotState` | Render optional controls, links, or retry UI. | none    |

## Expose

| Name              | Type                       | Purpose                                                      | Default         |
| ----------------- | -------------------------- | ------------------------------------------------------------ | --------------- |
| `element`         | `CalloutElement \| null`   | Rendered host element or component instance.                 | `null`          |
| `open`            | `boolean`                  | Visibility boolean.                                          | `true`          |
| `state`           | `"open" \| "closed"`       | Visibility state token.                                      | `"open"`        |
| `role`            | `CalloutRole`              | Requested accessibility role.                                | `"note"`        |
| `ariaState`       | `CalloutAriaState`         | Resolved accessibility policy; closed content is decorative. | `"note"`        |
| `live`            | `CalloutLive \| undefined` | Live-region politeness.                                      | `undefined`     |
| `atomic`          | `boolean`                  | Live-region atomicity.                                       | `true`          |
| `tone`            | `CalloutTone`              | Consumer styling tone token.                                 | `"neutral"`     |
| `density`         | `CalloutDensity`           | Consumer density token.                                      | `"comfortable"` |
| `titleId`         | `string \| undefined`      | Resolved title wrapper id.                                   | `undefined`     |
| `descriptionId`   | `string \| undefined`      | Resolved description wrapper id.                             | `undefined`     |
| `ariaLabelledby`  | `string \| undefined`      | Resolved root label ids.                                     | `undefined`     |
| `ariaDescribedby` | `string \| undefined`      | Resolved root description ids.                               | `undefined`     |
| `hasIcon`         | `boolean`                  | Whether the icon slot is present.                            | `false`         |
| `hasTitle`        | `boolean`                  | Whether the title slot is present.                           | `false`         |
| `hasDescription`  | `boolean`                  | Whether the description slot is present.                     | `false`         |
| `hasActions`      | `boolean`                  | Whether the actions slot is present.                         | `false`         |

## Data Attributes

| Attribute              | Values                                                                  | Purpose                         | Default         |
| ---------------------- | ----------------------------------------------------------------------- | ------------------------------- | --------------- |
| `data-vize-ui`         | `"callout"`                                                             | Stable family selector.         | always          |
| `data-state`           | `"open"`, `"closed"`                                                    | Visibility state.               | `"open"`        |
| `data-tone`            | `"accent"`, `"danger"`, `"info"`, `"neutral"`, `"success"`, `"warning"` | Consumer tone styling hook.     | `"neutral"`     |
| `data-density`         | `"compact"`, `"comfortable"`                                            | Consumer density hook.          | `"comfortable"` |
| `data-aria-state`      | `"decorative"`, `"note"`, `"status"`, `"alert"`                         | Accessibility policy hook.      | `"note"`        |
| `data-live`            | `"off"`, `"polite"`, `"assertive"`                                      | Live-region policy hook.        | `"off"`         |
| `data-has-icon`        | `"true"`, `"false"`                                                     | Icon slot presence hook.        | `"false"`       |
| `data-has-title`       | `"true"`, `"false"`                                                     | Title slot presence hook.       | `"false"`       |
| `data-has-description` | `"true"`, `"false"`                                                     | Description slot presence hook. | `"false"`       |
| `data-has-actions`     | `"true"`, `"false"`                                                     | Actions slot presence hook.     | `"false"`       |

## ARIA Attributes

| Attribute          | Values                          | Purpose                                                        | Default                  |
| ------------------ | ------------------------------- | -------------------------------------------------------------- | ------------------------ |
| `role`             | `"note"`, `"status"`, `"alert"` | Defines static or live feedback semantics.                     | `"note"`                 |
| `hidden`           | `true`                          | Keeps closed Callouts mounted but hidden.                      | `undefined`              |
| `aria-hidden`      | `"true"`                        | Hides closed or decorative Callouts from assistive technology. | `undefined`              |
| `aria-label`       | `string`                        | Optional direct accessible name.                               | `undefined`              |
| `aria-labelledby`  | `string`                        | External or title-slot accessible name.                        | generated title id       |
| `aria-describedby` | `string`                        | External or description-slot accessible description.           | generated description id |
| `aria-live`        | `"polite"` or `"assertive"`     | Status or alert live-region politeness.                        | `undefined`              |
| `aria-atomic`      | `"true"` or `"false"`           | Live-region atomicity.                                         | `undefined` for `"note"` |

## CSS Custom Properties

Callout defines no CSS custom properties and ships no stylesheet. Consumers own
layout, icon spacing, typography, action placement, color, forced-colors
treatment, and responsive composition through ordinary CSS.

## Parts

| Part          | Element | Purpose                                                    |
| ------------- | ------- | ---------------------------------------------------------- |
| `root`        | host    | Style the rendered Callout host.                           |
| `icon`        | `span`  | Style optional icon chrome.                                |
| `content`     | `div`   | Style title, description, body, and actions as one region. |
| `title`       | `div`   | Style the title slot wrapper.                              |
| `description` | `div`   | Style the description slot wrapper.                        |
| `actions`     | `div`   | Style optional action controls.                            |
