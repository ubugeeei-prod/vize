# ScrollArea behavior contract

Normative state x input -> outcome table for `scroll-area.vue`
(`@vizejs/ui/scroll-area`). Every row is proven by the named mounted-DOM, SSR,
runtime-conformance, renderer, or packaging test. A row without a passing test
is a contract violation.

| #   | State             | Input                          | Outcome                                                                                                                                                      | Proven by                                                                             |
| --- | ----------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| S1  | default           | render / Tab                   | renders an unlabelled `<div data-vize-ui="scroll-area">` with a native vertical viewport, no generated ids, no focus stop, and slotted focus order preserved | `renders a vertical native viewport by default without generating ids or focus stops` |
| S2  | sizing            | props                          | normalizes numeric lengths to px, keeps authored CSS strings intact, and publishes root CSS custom properties                                                | `resolves native overflow, sizing, and CSS hook state without DOM reads`              |
| S3  | orientation       | `vertical`/`horizontal`/`both` | resolves native `overflow-x`/`overflow-y` hooks and mirrors the axis through root and viewport data attributes                                               | `resolves native overflow, sizing, and CSS hook state without DOM reads`              |
| S4  | labelled viewport | ARIA props                     | normalizes typed ARIA strings, promotes the viewport to `role="region"` only when named, and omits generated ids                                             | `renders an RTL labelled region with native scrolling hooks`                          |
| S5  | keyboard viewport | `focusable=true`               | makes the native viewport focusable with `tabindex="0"` and exposes `focus()` without changing default tab order                                             | `emits native scroll events and exposes focus and scroll methods`                     |
| S6  | native scroll     | scroll event / methods         | forwards native scroll events and exposes `scrollTo()` / `scrollBy()` pass-through methods                                                                   | `emits native scroll events and exposes focus and scroll methods`                     |
| S7  | LTR/RTL           | `dir`                          | reflects explicit reading direction through native `dir` and `data-dir` on both root and viewport                                                            | `renders an RTL labelled region with native scrolling hooks`                          |
| S8  | custom root       | component host                 | renders consumer components while preserving ScrollArea parts, data hooks, style hooks, and viewport semantics                                               | `renders a consumer component root without dropping scroll hooks`                     |
| S9  | slot/expose       | update props                   | passes the complete resolved contract to the slot and exposes live root, viewport, ARIA, axis, size, and style state                                         | `passes slot state and exposes live viewport state`                                   |
| S10 | SSR labelled      | isolated requests              | renders byte-identical labelled server markup without request-global state                                                                                   | `renders byte-identical labelled scroll area markup across isolated SSR requests`     |
| S11 | SSR default       | render                         | omits optional ARIA and focus attributes from default server markup                                                                                          | `omits optional ARIA and focus attributes from default SSR markup`                    |
| S12 | SSR/hydration     | runtime fixture                | server markup hydrates without warnings or root node replacement                                                                                             | `runtime-conformance.test.ts`                                                         |
| S13 | DOM/SSR/Vapor     | compile                        | authored SFC compiles in every renderer lane without warnings or fallback                                                                                    | `scripts/check-renderers.ts`                                                          |
| S14 | root/subpath      | consumer bundle                | root and subpath consumers retain only ScrollArea and its native CSS hooks within gzip budgets                                                               | `scripts/check-tree-shaking.mjs`                                                      |

## Props

| Prop                 | Type                                        | Purpose                                                                     | Default      |
| -------------------- | ------------------------------------------- | --------------------------------------------------------------------------- | ------------ |
| `as`                 | `PrimitiveAs`                               | Native element, custom element, or component rendered as the root.          | `"div"`      |
| `orientation`        | `"vertical" \| "horizontal" \| "both"`      | Logical scroll axis mapped to native viewport overflow.                     | `"vertical"` |
| `dir`                | `"ltr" \| "rtl"`                            | Reading direction reflected with `dir` and `data-dir`.                      | `"ltr"`      |
| `focusable`          | `boolean`                                   | Adds `tabindex="0"` to the viewport for standalone keyboard scrolling.      | `false`      |
| `blockSize`          | `string \| number`                          | Root `block-size` hook; numbers resolve to px.                              | `"auto"`     |
| `inlineSize`         | `string \| number`                          | Root `inline-size` hook; numbers resolve to px.                             | `"auto"`     |
| `maxBlockSize`       | `string \| number`                          | Root `max-block-size` hook; numbers resolve to px.                          | `"none"`     |
| `maxInlineSize`      | `string \| number`                          | Root `max-inline-size` hook; numbers resolve to px.                         | `"none"`     |
| `overscrollBehavior` | `"auto" \| "contain" \| "none"`             | Native viewport overscroll policy.                                          | `"auto"`     |
| `scrollBehavior`     | `"auto" \| "smooth"`                        | Native viewport programmatic scroll behavior; reduced motion forces `auto`. | `"auto"`     |
| `scrollbarGutter`    | `"auto" \| "stable" \| "stable both-edges"` | Native viewport scrollbar gutter policy.                                    | `"auto"`     |
| `scrollbarWidth`     | `"auto" \| "thin" \| "none"`                | Native viewport scrollbar width hook.                                       | `"auto"`     |
| `ariaLabel`          | `string`                                    | Accessible name for the viewport; promotes it to `role="region"`.           | `undefined`  |
| `ariaLabelledby`     | `string`                                    | Space-separated ids that label the viewport.                                | `undefined`  |
| `ariaDescribedby`    | `string`                                    | Space-separated ids that describe the viewport.                             | `undefined`  |

## Emits

| Emit     | Payload                | Purpose                                                   | Default |
| -------- | ---------------------- | --------------------------------------------------------- | ------- |
| `scroll` | `[nativeEvent: Event]` | Fired when the native viewport dispatches a scroll event. | none    |

## Slots

| Slot      | Props                 | Purpose                                                                                                   | Default |
| --------- | --------------------- | --------------------------------------------------------------------------------------------------------- | ------- |
| `default` | `ScrollAreaSlotState` | Render consumer-owned scrollable content with resolved axis, direction, size, ARIA, and style-hook state. | none    |

## Expose

| Name                 | Type                                  | Purpose                                               | Default        |
| -------------------- | ------------------------------------- | ----------------------------------------------------- | -------------- |
| `root`               | `PrimitiveElement \| null`            | Rendered root element or component.                   | `null`         |
| `viewport`           | `HTMLDivElement \| null`              | Native scroll viewport.                               | `null`         |
| `focus`              | `(options?: FocusOptions) => void`    | Moves DOM focus to the viewport.                      | n/a            |
| `scrollTo`           | `(options?: ScrollToOptions) => void` | Pass-through to `viewport.scrollTo`.                  | n/a            |
| `scrollBy`           | `(options?: ScrollToOptions) => void` | Pass-through to `viewport.scrollBy`.                  | n/a            |
| `as`                 | `ScrollAreaAs`                        | Rendered root host.                                   | `"div"`        |
| `orientation`        | `ScrollAreaOrientation`               | Logical scroll axis.                                  | `"vertical"`   |
| `dir`                | `ScrollAreaDirection`                 | Reflected reading direction.                          | `"ltr"`        |
| `focusable`          | `boolean`                             | Whether the viewport receives `tabindex="0"`.         | `false`        |
| `blockSize`          | `string`                              | Resolved root block size.                             | `"auto"`       |
| `inlineSize`         | `string`                              | Resolved root inline size.                            | `"auto"`       |
| `maxBlockSize`       | `string`                              | Resolved root max block size.                         | `"none"`       |
| `maxInlineSize`      | `string`                              | Resolved root max inline size.                        | `"none"`       |
| `overflowX`          | `"auto" \| "hidden"`                  | Native viewport horizontal overflow hook.             | `"hidden"`     |
| `overflowY`          | `"auto" \| "hidden"`                  | Native viewport vertical overflow hook.               | `"auto"`       |
| `overscrollBehavior` | `ScrollAreaOverscrollBehavior`        | Native viewport overscroll policy.                    | `"auto"`       |
| `scrollBehavior`     | `ScrollAreaScrollBehavior`            | Native viewport scroll behavior.                      | `"auto"`       |
| `scrollbarGutter`    | `ScrollAreaScrollbarGutter`           | Native viewport scrollbar gutter policy.              | `"auto"`       |
| `scrollbarWidth`     | `ScrollAreaScrollbarWidth`            | Native viewport scrollbar width hook.                 | `"auto"`       |
| `ariaLabel`          | `string \| undefined`                 | Normalized viewport `aria-label`.                     | `undefined`    |
| `ariaLabelledby`     | `string \| undefined`                 | Normalized viewport `aria-labelledby`.                | `undefined`    |
| `ariaDescribedby`    | `string \| undefined`                 | Normalized viewport `aria-describedby`.               | `undefined`    |
| `labelled`           | `boolean`                             | Whether the viewport has an accessible name.          | `false`        |
| `described`          | `boolean`                             | Whether a description IDREF is rendered.              | `false`        |
| `state`              | `"scrollable"`                        | Stable state token.                                   | `"scrollable"` |
| `style`              | `ScrollAreaStyle`                     | Native CSS custom property hooks applied to the root. | see CSS vars   |

## Data Attributes

| Attribute                  | Host           | Values                                 | Purpose                          | Default      |
| -------------------------- | -------------- | -------------------------------------- | -------------------------------- | ------------ |
| `data-vize-ui`             | root           | `"scroll-area"`                        | Stable family selector.          | always       |
| `data-vize-ui`             | viewport       | `"scroll-area-viewport"`               | Stable viewport selector.        | always       |
| `data-state`               | root, viewport | `"scrollable"`                         | Stable state hook.               | always       |
| `data-orientation`         | root, viewport | `"vertical"`, `"horizontal"`, `"both"` | Logical axis hook.               | `"vertical"` |
| `data-dir`                 | root, viewport | `"ltr"`, `"rtl"`                       | Reading direction hook.          | `"ltr"`      |
| `data-focusable`           | root, viewport | `"true"`, `"false"`                    | Keyboard focusability hook.      | `"false"`    |
| `data-overflow-x`          | viewport       | `"auto"`, `"hidden"`                   | Native horizontal overflow hook. | derived      |
| `data-overflow-y`          | viewport       | `"auto"`, `"hidden"`                   | Native vertical overflow hook.   | derived      |
| `data-overscroll-behavior` | root           | `ScrollAreaOverscrollBehavior`         | Native overscroll hook.          | `"auto"`     |
| `data-scroll-behavior`     | root           | `ScrollAreaScrollBehavior`             | Native scroll-behavior hook.     | `"auto"`     |
| `data-scrollbar-gutter`    | root           | `ScrollAreaScrollbarGutter`            | Native scrollbar-gutter hook.    | `"auto"`     |
| `data-scrollbar-width`     | root           | `ScrollAreaScrollbarWidth`             | Native scrollbar-width hook.     | `"auto"`     |

## CSS Custom Properties

| Property                                    | Host | Purpose                                                                       | Default  |
| ------------------------------------------- | ---- | ----------------------------------------------------------------------------- | -------- |
| `--vize-ui-scroll-area-block-size`          | root | Root `block-size`.                                                            | `"auto"` |
| `--vize-ui-scroll-area-inline-size`         | root | Root `inline-size`.                                                           | `"auto"` |
| `--vize-ui-scroll-area-max-block-size`      | root | Root `max-block-size`.                                                        | `"none"` |
| `--vize-ui-scroll-area-max-inline-size`     | root | Root `max-inline-size`.                                                       | `"none"` |
| `--vize-ui-scroll-area-overflow-x`          | root | Viewport `overflow-x`.                                                        | derived  |
| `--vize-ui-scroll-area-overflow-y`          | root | Viewport `overflow-y`.                                                        | derived  |
| `--vize-ui-scroll-area-overscroll-behavior` | root | Viewport `overscroll-behavior`.                                               | `"auto"` |
| `--vize-ui-scroll-area-scroll-behavior`     | root | Viewport `scroll-behavior`; reduced motion overrides to `auto`.               | `"auto"` |
| `--vize-ui-scroll-area-scrollbar-gutter`    | root | Viewport `scrollbar-gutter`.                                                  | `"auto"` |
| `--vize-ui-scroll-area-scrollbar-width`     | root | Viewport `scrollbar-width`; forced colors restores system scrollbar handling. | `"auto"` |

## Parts

| Part       | Purpose                         | Default |
| ---------- | ------------------------------- | ------- |
| `root`     | Root size and family hook host. | always  |
| `viewport` | Native scroll container.        | always  |
