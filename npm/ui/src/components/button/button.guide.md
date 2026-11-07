## Design the action before the surface

Use a native `<button>` whenever the action can be expressed by one. `Button`
keeps the platform's Enter, Space, form, focus, and disabled behavior while
adding one typed `press` event. Use `as` only when composition requires another
element or component, and set `native` to describe whether that target already
implements button semantics.

```vue
<script setup lang="ts">
import { Button } from "@vizejs/ui/button";
</script>

<template>
  <Button type="submit">Save profile</Button>
  <Button type="button" variant="outline" tone="neutral">Preview</Button>
  <Button type="button" variant="quiet" tone="danger">Delete</Button>
</template>
```

`press` always carries the activation `MouseEvent`, whether the user activates
with a pointer, Enter, or Space. That gives an application one handler without
erasing native event timing.

## Model unavailable states deliberately

`disabled` and `loading` are different product states:

| State      | Focus | Activation | Accessible signal                         |
| ---------- | ----- | ---------- | ----------------------------------------- |
| available  | yes   | emits once | native button semantics                   |
| `loading`  | yes   | suppressed | `aria-busy` and `aria-disabled`           |
| `disabled` | no    | suppressed | native `disabled`, or ARIA for non-native |

Keeping a loading action focusable prevents focus from disappearing while an
operation is in flight. Use the typed slot state to keep its label truthful:

```vue
<Button v-slot="{ loading }" type="button" :loading="saving" @press="save">
  {{ loading ? "Saving…" : "Save changes" }}
</Button>
```

## Choose a visual voice, then customize it

Behavior is headless. The optional stylesheet consumes only semantic data
attributes and public custom properties:

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
```

```html
<main data-vize-theme="paper">…</main>
```

`variant`, `tone`, and `size` are closed TypeScript unions and stable styling
hooks. Presets can change geometry, typography, depth, and motion without
changing the component contract. Override the component through its public
properties instead of matching implementation structure:

```css
.checkout-actions {
  --vize-ui-button-radius: 0.5rem;
  --vize-ui-button-shadow: none;
  --vize-ui-button-hover-lift: 0px;

  [data-vize-ui="button"] {
    letter-spacing: 0.01em;
  }
}
```

The shipped CSS is authored with native nesting and lowered with Lightning CSS.
Consumers receive ordinary CSS and do not need Tailwind, a CSS-in-JS runtime,
or the Vize build pipeline. Application CSS outside Vize's cascade layers wins
normally. See [UI styles](../ui-styles.md) for every preset and token boundary.

## Accessibility is policy, not a preset

Every visual voice retains a visible `:focus-visible` ring, a 44 CSS-pixel
minimum target on coarse pointers, forced-colors mappings, and a
`prefers-reduced-motion` path. The loading indicator stops animating under
reduced motion while the textual state remains available.

An icon-only button still needs an accessible name. Provide `aria-label` or
visible text; the component does not guess a label from an icon.

## Render it in any supported Vue host

The SFC has no browser-only setup path. The same public import is used for DOM,
SSR, hydration, and Vapor compilation. Keep the Vue runtime, compiler, and
server renderer on a compatible version when using Vapor; see the
[Vapor guide](../vapor.md) for the currently qualified pairing.

| Host        | Use                                                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------------------------------------ |
| Vue / Vite+ | Import the component subpath in any SFC.                                                                                 |
| Nuxt        | Import it directly, or enable the `@vizejs/nuxt` UI auto-registration. See [Nuxt](../../integrations/nuxt.md).           |
| Void        | Import it in an `@void/vue` page; component CSS stays in Void's route CSS graph. See [Void](../../integrations/void.md). |

For maximum ownership, `vize lib pull button` copies the SFC unit into the
application. The public package import and the source-distributed form share the
same behavior contract and tests.

## Bundle only what the screen uses

Import from `@vizejs/ui/button`, not the package root, when the screen needs only
Button. Its JavaScript budget is enforced on a production consumer, and the
visual CSS is a separate opt-in asset. Importing the behavior never silently
adds a theme or another component's stylesheet.
