# Spinner behavior contract

Normative state x input -> outcome table for `spinner.vue` (`@vizejs/ui/spinner`
sidecar). Every row is proven by the named mounted-DOM, SSR/hydration, or
compile-only type test. A row without a passing test is a contract violation.

| #   | State              | Input             | Outcome                                                                                           | Proven by                                                               |
| --- | ------------------ | ----------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| S1  | default status     | render            | renders a headless `<span role="status">` with a deterministic id, polite live region, and no tab | `renders a polite status spinner by default`                            |
| S2  | progressbar        | determinate value | exposes normalized `aria-valuenow/min/max`, value text, progress data attrs, and no live region   | `renders progressbar semantics with normalized determinate values`      |
| S3  | progressbar        | out-of-range      | clamps finite values to the normalized range and marks complete work with `data-state="complete"` | `renders progressbar semantics with normalized determinate values`      |
| S4  | progressbar        | unknown value     | omits `aria-valuenow`, `data-value`, and `data-percent` while marking indeterminate progress      | `renders progressbar semantics with normalized determinate values`      |
| S5  | decorative         | `ariaHidden`      | suppresses role, label, description, and progress ARIA while retaining observable data state      | `lets ariaHidden make labelled progress spinners decorative`            |
| S6  | visibility/loading | prop update       | keeps the host mounted while updating `hidden`, slot state, and exposed state                     | `updates visibility, loading, slot state, and exposed state`            |
| S7  | named explicit id  | render            | honors a consumer id and `aria-labelledby` naming without generating a replacement id             | `honors explicit ids and labelledby naming`                             |
| S8  | SSR status         | isolated requests | renders byte-identical generated-id status markup with no request-global state                    | `renders byte-identical status markup across isolated SSR requests`     |
| S9  | hydration          | generated id      | hydrates in place with the same generated id and no diagnostics                                   | `hydrates generated ids without replacing the spinner root`             |
| S10 | SSR progressbar    | render            | renders determinate server progressbar markup without status live-region attributes               | `renders determinate progressbar markup without live-region attributes` |
| S11 | public types       | invalid contract  | TypeScript rejects unsupported roles, state tokens, value types, and malformed slot state         | `src/spinner.types.test-d.ts`                                           |

## Props

| Prop              | Type                        | Purpose                                                                    | Default     |
| ----------------- | --------------------------- | -------------------------------------------------------------------------- | ----------- |
| `as`              | `PrimitiveAs`               | Native element, custom element, or component rendered as host.             | `"span"`    |
| `id`              | `string \| null`            | Consumer-owned id; `null` and `undefined` select a deterministic fallback. | `undefined` |
| `loading`         | `boolean`                   | Whether the spinner represents pending work.                               | `true`      |
| `visible`         | `boolean`                   | Whether the host remains visible in layout.                                | `true`      |
| `role`            | `"status" \| "progressbar"` | Accessibility semantics used unless `ariaHidden` is true.                  | `"status"`  |
| `value`           | `number \| null`            | Optional determinate progress value for `role="progressbar"`.              | `null`      |
| `min`             | `number`                    | Lower progress bound for `role="progressbar"`.                             | `0`         |
| `max`             | `number`                    | Upper progress bound for `role="progressbar"`.                             | `100`       |
| `atomic`          | `boolean`                   | Whether status announcements should be atomic.                             | `true`      |
| `ariaHidden`      | `boolean`                   | Forces decorative semantics and suppresses status/progress ARIA.           | `undefined` |
| `ariaLabel`       | `string`                    | Accessible name when no visible label or `aria-labelledby` supplies one.   | `undefined` |
| `ariaLabelledby`  | `string`                    | Space-separated ids that label the spinner.                                | `undefined` |
| `ariaDescribedby` | `string`                    | Space-separated ids that describe the spinner.                             | `undefined` |
| `ariaValueText`   | `string`                    | Human-readable progress text for `role="progressbar"`.                     | `undefined` |

## Slots

| Slot      | Props              | Purpose                                       | Default |
| --------- | ------------------ | --------------------------------------------- | ------- |
| `default` | `SpinnerSlotState` | Render optional spinner glyph or status copy. | none    |

## Expose

| Name            | Type                     | Purpose                                   | Default     |
| --------------- | ------------------------ | ----------------------------------------- | ----------- |
| `element`       | `SpinnerElement \| null` | Rendered host element or component.       | `null`      |
| `loading`       | `boolean`                | Whether pending work is represented.      | `true`      |
| `visible`       | `boolean`                | Whether the host is not hidden.           | `true`      |
| `state`         | `SpinnerState`           | Current visibility/loading state token.   | `"loading"` |
| `ariaState`     | `SpinnerAriaState`       | Resolved accessibility policy.            | `"status"`  |
| `progressState` | `SpinnerProgressState`   | Whether progress values are exposed.      | `"none"`    |
| `value`         | `number \| null`         | Current normalized progress value.        | `null`      |
| `min`           | `number`                 | Current normalized progress lower bound.  | `0`         |
| `max`           | `number`                 | Current normalized progress upper bound.  | `100`       |
| `percent`       | `number \| null`         | Current completion percentage.            | `null`      |
| `complete`      | `boolean`                | Whether determinate progress is complete. | `false`     |

## Data Attributes

| Attribute             | Values                                          | Purpose                          | Default     |
| --------------------- | ----------------------------------------------- | -------------------------------- | ----------- |
| `data-vize-ui`        | `"spinner"`                                     | Stable family selector.          | always      |
| `data-state`          | `"complete"`, `"hidden"`, `"idle"`, `"loading"` | Visibility and loading state.    | `"loading"` |
| `data-loading`        | `"true"`, `"false"`                             | Boolean loading styling hook.    | `"true"`    |
| `data-visible`        | `"true"`, `"false"`                             | Boolean visibility styling hook. | `"true"`    |
| `data-aria-state`     | `"decorative"`, `"progressbar"`, `"status"`     | Accessibility policy hook.       | `"status"`  |
| `data-progress-state` | `"determinate"`, `"indeterminate"`, `"none"`    | Progress value policy hook.      | `"none"`    |
| `data-complete`       | `"true"`, `"false"`                             | Determinate completion hook.     | `"false"`   |
| `data-value`          | `number`                                        | Normalized progress value.       | `undefined` |
| `data-min`            | `number`                                        | Normalized progress lower bound. | `undefined` |
| `data-max`            | `number`                                        | Normalized progress upper bound. | `undefined` |
| `data-percent`        | `number`                                        | Normalized completion percent.   | `undefined` |

## ARIA Attributes

| Attribute          | Values                        | Purpose                                           | Default     |
| ------------------ | ----------------------------- | ------------------------------------------------- | ----------- |
| `role`             | `"status"` or `"progressbar"` | Announces status or progress semantics.           | `"status"`  |
| `aria-hidden`      | `"true"`                      | Hides decorative spinners from assistive tech.    | `undefined` |
| `aria-label`       | `string`                      | Optional accessible name.                         | `undefined` |
| `aria-labelledby`  | `string`                      | Optional external accessible name.                | `undefined` |
| `aria-describedby` | `string`                      | Optional external accessible description.         | `undefined` |
| `aria-live`        | `"polite"`                    | Status live-region politeness.                    | `"polite"`  |
| `aria-atomic`      | `"true"` or `"false"`         | Status live-region atomicity.                     | `"true"`    |
| `aria-valuemin`    | `number`                      | Progress lower bound.                             | `undefined` |
| `aria-valuemax`    | `number`                      | Progress upper bound.                             | `undefined` |
| `aria-valuenow`    | `number`                      | Current determinate progress value.               | `undefined` |
| `aria-valuetext`   | `string`                      | Human-readable progress value.                    | `undefined` |
| `hidden`           | present or undefined          | Hides the host without unmounting when invisible. | `undefined` |

## Parts

| Part   | Element | Purpose                          |
| ------ | ------- | -------------------------------- |
| `root` | host    | Style the rendered Spinner host. |

## Styling Contract

Spinner is headless: it emits no visual CSS, animation, SVG, or color preset.
Consumers provide the glyph and motion through the default slot or CSS using the
root part and stable data attributes.
