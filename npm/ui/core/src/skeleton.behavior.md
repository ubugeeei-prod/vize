# Skeleton behavior contract

Normative state x input -> outcome table for `skeleton.vue` (`@vizejs/ui/skeleton`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State            | Input             | Outcome                                                                                                 | Proven by                                                                    |
| --- | ---------------- | ----------------- | ------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| K1  | default          | render            | renders `<div data-vize-ui="skeleton">`, `data-state="loading"`, `part="root"`, CSS hooks, and no focus | `renders a decorative loading placeholder by default`                        |
| K2  | labelled loading | render            | renders the requested host as `role="status"` with `aria-label` and stable styling hooks                | `renders status semantics when labelled`                                     |
| K3  | forced hidden AT | render            | `ariaHidden` suppresses status role and label even when a label is supplied                             | `lets ariaHidden override labelled status semantics`                         |
| K4  | loaded/hidden    | prop update       | keeps the host mounted while updating `hidden`, `data-state`, `data-loading`, and `data-visible`        | `keeps hidden and loaded states observable without unmounting`               |
| K5  | any              | slot/expose       | passes loading, visibility, state, and ARIA policy to the slot and exposed component instance           | `passes slot state and exposes live element/loading state`                   |
| K6  | SSR status       | isolated requests | renders byte-identical labelled status markup with the same data and style hooks                        | `renders byte-identical status skeleton markup across isolated SSR requests` |
| K7  | SSR decorative   | render            | renders decorative hidden markup without status role or label                                           | `renders decorative server markup without status ARIA`                       |
| K8  | DOM/SSR/Vapor    | compile           | authored SFC compiles in every renderer lane without warnings or fallback                               | `scripts/check-renderers.ts`                                                 |
| K9  | root/subpath     | consumer bundle   | root and subpath consumers retain only Skeleton, emit no CSS, and stay within gzip budget               | `scripts/check-tree-shaking.mjs`                                             |

## Props

| Prop         | Type          | Purpose                                                                      | Default     |
| ------------ | ------------- | ---------------------------------------------------------------------------- | ----------- |
| `as`         | `PrimitiveAs` | Native element, custom element, or component rendered as host.               | `"div"`     |
| `loading`    | `boolean`     | Whether the placeholder represents pending content.                          | `true`      |
| `visible`    | `boolean`     | Whether the placeholder remains rendered and visible in layout.              | `true`      |
| `ariaLabel`  | `string`      | Accessible status text when the skeleton should be announced.                | `undefined` |
| `ariaHidden` | `boolean`     | Override the derived accessibility policy. `true` makes the host decorative. | `undefined` |
| `blockSize`  | `string`      | Value published to `--vize-ui-skeleton-block-size`.                          | `"1em"`     |
| `inlineSize` | `string`      | Value published to `--vize-ui-skeleton-inline-size`.                         | `"100%"`    |

## Slots

| Slot      | Props                                                                                        | Purpose                                         | Default |
| --------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------- | ------- |
| `default` | `{ loading: boolean; visible: boolean; state: SkeletonState; ariaState: SkeletonAriaState }` | Render optional placeholder content or markers. | none    |

## Expose

| Name        | Type                       | Purpose                                        | Default        |
| ----------- | -------------------------- | ---------------------------------------------- | -------------- |
| `element`   | `SkeletonElement \| null`  | Rendered host element or component instance.   | `null`         |
| `loading`   | `boolean`                  | Whether pending content is represented.        | `true`         |
| `visible`   | `boolean`                  | Whether the host is not hidden.                | `true`         |
| `state`     | `SkeletonState`            | Current visual state token.                    | `"loading"`    |
| `ariaState` | `"decorative" \| "status"` | Derived accessibility policy used by the host. | `"decorative"` |

## Data Attributes

| Attribute         | Values                              | Purpose                            | Default        |
| ----------------- | ----------------------------------- | ---------------------------------- | -------------- |
| `data-vize-ui`    | `"skeleton"`                        | Stable family selector.            | always         |
| `data-state`      | `"hidden"`, `"loaded"`, `"loading"` | Visibility and loading state.      | `"loading"`    |
| `data-loading`    | `"true"`, `"false"`                 | Boolean loading styling hook.      | `"true"`       |
| `data-visible`    | `"true"`, `"false"`                 | Boolean visibility styling hook.   | `"true"`       |
| `data-aria-state` | `"decorative"`, `"status"`          | Accessibility policy styling hook. | `"decorative"` |

## ARIA Attributes

| Attribute     | Values               | Purpose                                                                  | Default     |
| ------------- | -------------------- | ------------------------------------------------------------------------ | ----------- |
| `role`        | `"status"`           | Announces a labelled skeleton as a polite status region.                 | `undefined` |
| `aria-hidden` | `"true"`             | Hides decorative skeletons from assistive technology.                    | `"true"`    |
| `aria-label`  | `string`             | Optional status name when the skeleton is announced.                     | `undefined` |
| `hidden`      | present or undefined | Hides the rendered host without unmounting it when `visible` is `false`. | `undefined` |

## CSS Custom Properties

| Custom property                  | Purpose                                       | Default  |
| -------------------------------- | --------------------------------------------- | -------- |
| `--vize-ui-skeleton-block-size`  | Consumer styling hook for placeholder height. | `"1em"`  |
| `--vize-ui-skeleton-inline-size` | Consumer styling hook for placeholder width.  | `"100%"` |

## Parts

| Part   | Element | Purpose                           |
| ------ | ------- | --------------------------------- |
| `root` | host    | Style the rendered Skeleton host. |

## Styling Contract

Skeleton is headless: it emits no stylesheet, animation, shimmer, or color
preset. Consumers opt into visual treatment with ordinary CSS using the root
part, data attributes, and the published custom properties.
