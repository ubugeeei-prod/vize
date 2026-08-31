# Banner behavior contract

Normative state x input -> outcome table for `banner.vue`
(`@vizejs/ui/banner` future sidecar). Every row is proven by the named
mounted-DOM, SSR/hydration, runtime-helper, or compile-only type test. A row
without a passing test is a contract violation.

| #   | State          | Input             | Outcome                                                                                             | Proven by                                                                        |
| --- | -------------- | ----------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| B1  | named region   | title/description | renders a headless `<section role="region">` named by a deterministic title id                      | `renders a named persistent region with deterministic title and description ids` |
| B2  | aria override  | label/describedby | prefers explicit ARIA labels and merges external plus deterministic description ids                 | `normalizes explicit ARIA labels and external descriptions before title ids`     |
| B3  | live roles     | status/alert      | maps status to polite and alert to assertive live-region attributes with atomic policy              | `supports status and alert live-role banners`                                    |
| B4  | unnamed region | empty name        | suppresses the region role rather than emitting an unnamed landmark                                 | `suppresses an unnamed region role instead of emitting an unnamed landmark`      |
| B5  | dismissible    | click             | emits controlled `update:open=false` and `dismiss` without mutating local visibility                | `requests controlled dismissal without mutating local visibility`                |
| B6  | closed         | `open=false`      | keeps deterministic DOM, hides content natively, and suppresses ARIA semantics while exposing state | `hides closed banners and exposes closed state`                                  |
| B7  | custom host    | `as` component    | forwards root attributes, parts, ARIA, and slots through a consumer component                       | `forwards root attributes and slots through a custom as component`               |
| B8  | runtime helper | normalize         | trims names/idrefs, applies precedence, and resolves named/unnamed/live states deterministically    | `normalizes banner aria as a standalone runtime helper`                          |
| B9  | SSR region     | isolated requests | renders byte-identical named region markup with no class, style, tab, or handler leak               | `renders byte-identical named region markup across isolated SSR requests`        |
| B10 | hydration      | server markup     | hydrates the server-rendered host in place with no diagnostics                                      | `hydrates named SSR markup without replacing the banner root`                    |
| B11 | public types   | invalid contract  | TypeScript rejects unsupported roles, tones, states, region names, emits, and malformed slot state  | `src/families/feedback/banner/banner.types.test-d.ts`                            |

## Props

| Prop              | Type                                                                    | Purpose                                                                  | Default                                  |
| ----------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------- |
| `as`              | `PrimitiveAs`                                                           | Native element, custom element, or component rendered as host.           | `"section"`                              |
| `id`              | `string`                                                                | Optional root id used to derive title and description ids.               | generated                                |
| `title`           | `string`                                                                | Visible title and default accessible name.                               | required by region types unless labelled |
| `description`     | `string`                                                                | Visible description linked through `aria-describedby`.                   | `undefined`                              |
| `role`            | `"region" \| "status" \| "alert"`                                       | Persistent banner role; default regions require a name.                  | `"region"`                               |
| `tone`            | `"accent" \| "danger" \| "info" \| "neutral" \| "success" \| "warning"` | Consumer styling tone mirrored to `data-tone`.                           | `"neutral"`                              |
| `open`            | `boolean`                                                               | Controlled visibility; false sets `hidden` and `aria-hidden`.            | `true`                                   |
| `dismissible`     | `boolean`                                                               | Render a native dismiss control that requests `open=false`.              | `false`                                  |
| `dismissLabel`    | `string`                                                                | Accessible label and text for the dismiss control.                       | `"Dismiss banner"`                       |
| `atomic`          | `boolean`                                                               | Whether `status`/`alert` announcements are atomic.                       | `true`                                   |
| `ariaLabel`       | `string`                                                                | Accessible name when no visible title or `aria-labelledby` supplies one. | `undefined`                              |
| `ariaLabelledby`  | `string`                                                                | Space-separated ids that label the banner.                               | `undefined`                              |
| `ariaDescribedby` | `string`                                                                | Space-separated ids that describe the banner.                            | `undefined`                              |

## Slots

| Slot          | Props             | Purpose                                             | Default            |
| ------------- | ----------------- | --------------------------------------------------- | ------------------ |
| `default`     | `BannerSlotState` | Render the primary banner body.                     | none               |
| `title`       | `BannerSlotState` | Render a consumer-owned title in the title part.    | `title` prop       |
| `description` | `BannerSlotState` | Render a consumer-owned description in description. | `description` prop |
| `actions`     | `BannerSlotState` | Render trailing actions.                            | none               |

## Expose

| Name              | Type                                 | Purpose                                  | Default                       |
| ----------------- | ------------------------------------ | ---------------------------------------- | ----------------------------- |
| `element`         | `BannerElement \| null`              | Rendered host element or component.      | `null` before mount           |
| `focus`           | `(options?: FocusOptions) => void`   | Focus the rendered host when focusable.  | no-op while closed            |
| `dismiss`         | `(nativeEvent?: MouseEvent) => void` | Request controlled dismissal.            | emits only                    |
| `state`           | `"open" \| "closed"`                 | Controlled visibility state.             | `"open"`                      |
| `role`            | `BannerRole`                         | Persistent banner role.                  | `"region"`                    |
| `tone`            | `BannerTone`                         | Consumer styling tone token.             | `"neutral"`                   |
| `live`            | `BannerLive`                         | Resolved live-region politeness.         | `"off"`                       |
| `named`           | `boolean`                            | Whether the rendered host has a name.    | title/label-derived           |
| `ariaState`       | `BannerAriaState`                    | Resolved accessibility quality.          | `"named"` when titled         |
| `titleId`         | `string`                             | Deterministic title id.                  | generated                     |
| `descriptionId`   | `string`                             | Deterministic description id.            | generated                     |
| `ariaLabelledby`  | `string \| undefined`                | Resolved labelledby references.          | title id when titled          |
| `ariaDescribedby` | `string \| undefined`                | Resolved describedby references.         | description id when described |
| `dismissible`     | `boolean`                            | Whether the dismiss control is rendered. | `false`                       |

## Styling Contract

Banner is headless: it emits no visual CSS, animation, SVG, color preset, or
size preset. Consumers provide all pixels through slots or CSS using the root,
title, description, and dismiss parts plus stable data attributes.
