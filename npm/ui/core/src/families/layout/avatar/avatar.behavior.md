# Avatar behavior contract

Normative state x input -> outcome table for `avatar.vue` (`@vizejs/ui/avatar`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State              | Input             | Outcome                                                                                                  | Proven by                                                                     |
| --- | ------------------ | ----------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| A1  | default fallback   | render / Tab      | renders a headless root and fallback part with strict missing/present hooks, no ARIA, no focus, no style | `renders fallback content by default without adding semantics or styling`     |
| A2  | image source       | render / load     | renders a native image part with consumer-provided native image attributes and no fallback part          | `renders native image semantics and forwards load events`                     |
| A3  | unsafe source      | render            | renders fallback and reports `data-image=missing` rather than forwarding script-capable image sources    | `renders fallback for unsafe image sources without forwarding src`            |
| A4  | image failure      | error event       | emits `error`, switches to fallback, keeps source/status hooks inspectable, and preserves root attrs     | `switches failed images to fallback while keeping consumer attrs on the root` |
| A5  | source replacement | prop update       | resets image failure state when `src` changes and renders the image branch again                         | `switches failed images to fallback while keeping consumer attrs on the root` |
| A6  | any                | slot/expose       | passes state, source, fallback, name, and status hooks to slots and exposes live rendered parts          | `passes slot state and exposes live avatar state`                             |
| A7  | SSR fallback       | isolated requests | renders byte-identical fallback markup without request-global state                                      | `renders byte-identical fallback markup across isolated SSR requests`         |
| A8  | SSR image          | render            | renders stable image markup with native image attributes and no fallback part                            | `renders server image markup with native image attributes`                    |
| A9  | DOM/SSR/Vapor      | compile           | authored SFC compiles in every renderer lane without warnings or fallback                                | `scripts/check-renderers.ts`                                                  |
| A10 | SSR/hydration      | hydrate           | server avatar markup hydrates without warnings, node replacement, or accessibility drift                 | `src/runtime-conformance.test.ts`                                             |
| A11 | root/subpath       | consumer bundle   | root and subpath consumers retain only Avatar, emit no CSS, and stay within gzip budget                  | `scripts/check-tree-shaking.mjs`                                              |

## Props

| Prop             | Type                                                                                                              | Purpose                                                                | Default     |
| ---------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ----------- |
| `as`             | `PrimitiveAs`                                                                                                     | Native element, custom element, or component root host.                | `"span"`    |
| `src`            | `string`                                                                                                          | Safe native image source; missing, failed, or unsafe sources fallback. | `undefined` |
| `alt`            | `string`                                                                                                          | Native image alternative text.                                         | `""`        |
| `name`           | `string`                                                                                                          | Consumer-owned display name exposed to slots and hooks.                | `undefined` |
| `fallback`       | `string`                                                                                                          | Consumer-owned fallback text; never generated from name.               | `undefined` |
| `status`         | `"away" \| "busy" \| "none" \| "offline" \| "online"`                                                             | Consumer presence token mirrored to `data-status`.                     | `"none"`    |
| `loading`        | `"eager" \| "lazy"`                                                                                               | Native image loading policy.                                           | `undefined` |
| `decoding`       | `"async" \| "auto" \| "sync"`                                                                                     | Native image decoding policy.                                          | `undefined` |
| `fetchPriority`  | `"auto" \| "high" \| "low"`                                                                                       | Native image fetch-priority hint.                                      | `undefined` |
| `crossOrigin`    | `"" \| "anonymous" \| "use-credentials"`                                                                          | Native image CORS policy.                                              | `undefined` |
| `referrerPolicy` | `"no-referrer" \| "no-referrer-when-downgrade" \| "origin" \| "origin-when-cross-origin" \| "same-origin" \| ...` | Native image referrer policy.                                          | `undefined` |

## Slots

| Slot       | Props             | Purpose                                                | Default      |
| ---------- | ----------------- | ------------------------------------------------------ | ------------ |
| `default`  | `AvatarSlotState` | Render fallback content when `fallback` slot is empty. | `fallback`   |
| `fallback` | `AvatarSlotState` | Render named fallback content.                         | default slot |

## Emits

| Event   | Payload | Purpose                                                     |
| ------- | ------- | ----------------------------------------------------------- |
| `load`  | `Event` | Fired after the image part dispatches a native load event.  |
| `error` | `Event` | Fired after the image part dispatches a native error event. |

## Expose

| Name              | Type                            | Purpose                                      | Default      |
| ----------------- | ------------------------------- | -------------------------------------------- | ------------ |
| `element`         | `AvatarElement \| null`         | Rendered root element or component instance. | `null`       |
| `imageElement`    | `AvatarImageElement \| null`    | Rendered native image part.                  | `null`       |
| `fallbackElement` | `AvatarFallbackElement \| null` | Rendered native fallback part.               | `null`       |
| `state`           | `AvatarState`                   | Current render branch.                       | `"fallback"` |
| `status`          | `AvatarStatus`                  | Current presence token.                      | `"none"`     |
| `src`             | `string \| undefined`           | Current non-empty image source.              | `undefined`  |
| `alt`             | `string`                        | Current native image alt text.               | `""`         |
| `name`            | `string \| undefined`           | Current consumer display name.               | `undefined`  |
| `fallback`        | `string \| undefined`           | Current consumer fallback text.              | `undefined`  |
| `image`           | `AvatarPresence`                | Whether an image source is present.          | `"missing"`  |
| `nameState`       | `AvatarPresence`                | Whether a name is present.                   | `"missing"`  |
| `fallbackState`   | `AvatarPresence`                | Whether fallback content is present.         | `"missing"`  |

## Data Attributes

| Attribute       | Values                                                | Purpose                          | Default      |
| --------------- | ----------------------------------------------------- | -------------------------------- | ------------ |
| `data-vize-ui`  | `"avatar"`                                            | Stable family selector.          | always       |
| `data-state`    | `"image"`, `"fallback"`                               | Current rendered content branch. | `"fallback"` |
| `data-status`   | `"away"`, `"busy"`, `"none"`, `"offline"`, `"online"` | Consumer presence hook.          | `"none"`     |
| `data-image`    | `"missing"`, `"present"`                              | Image source presence hook.      | `"missing"`  |
| `data-name`     | `"missing"`, `"present"`                              | Name presence hook.              | `"missing"`  |
| `data-fallback` | `"missing"`, `"present"`                              | Fallback content presence hook.  | `"missing"`  |

The image part renders `data-vize-ui="avatar-image"` and the fallback part
renders `data-vize-ui="avatar-fallback"`.

## ARIA Attributes

Avatar never sets `role`, `tabindex`, `aria-hidden`, `aria-live`,
`aria-label`, or `aria-labelledby` by default. The image part receives the
native `alt` attribute and defaults it to an empty string so decorative avatar
images do not expose file names. Consumers that need labelled groups, live
presence updates, or hidden decorative roots pass ordinary Vue fallthrough
attributes to the root themselves.

## CSS Custom Properties

Avatar defines no CSS custom properties and ships no stylesheet. Consumers own
image sizing, object fitting, clipping, fallback layout, typography, and
presence indicator styling through ordinary CSS.

## Parts

| Part       | Purpose                    | Default                    |
| ---------- | -------------------------- | -------------------------- |
| `root`     | Single rendered root host. | always                     |
| `image`    | Native `<img>` part.       | when `data-state=image`    |
| `fallback` | Native `<span>` part.      | when `data-state=fallback` |
