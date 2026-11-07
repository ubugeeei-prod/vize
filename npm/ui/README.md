# @vizejs/ui

Accessible Vue SFCs with headless behavior, optional native CSS, and an escape
hatch to source ownership. The package does not require Tailwind, a CSS-in-JS
runtime, or a component plugin.

> `@vizejs/ui` is under active development. The catalogue is broad, but the
> component-unit architecture described below is being adopted incrementally.
> Check each generated reference page's maturity and quality gates before
> choosing it for a production boundary.

## Install

```bash
vp install @vizejs/ui
```

`pnpm add @vizejs/ui`, `npm install @vizejs/ui`, and `yarn add @vizejs/ui` work
as usual.

## Add one component

Imports are component-scoped so a screen does not retain the whole catalogue:

```vue
<script setup lang="ts">
import { ref } from "vue";

import { Button } from "@vizejs/ui/button";

const saves = ref(0);
</script>

<template>
  <Button type="button" @press="saves += 1">Save draft</Button>
  <output>Saved {{ saves }} times</output>
</template>
```

This is already complete behavior. It uses native button activation, produces
typed props and events, and renders without a visual stylesheet.

## Add a visual voice

Load the base tokens, one preset, and only the component styles the screen
uses:

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button type="button" variant="solid" tone="accent" size="md"> Save changes </Button>
  </main>
</template>
```

Seven presets ship as independent CSS assets:

| Preset          | Visual direction                                     |
| --------------- | ---------------------------------------------------- |
| `atelier`       | quiet studio neutrals and restrained depth           |
| `paper`         | warm ink, fine rules, squared editorial controls     |
| `signal`        | compact graphite, crisp edges, and monospace details |
| `midnight`      | luminous dark surfaces with controlled depth         |
| `play`          | rounded, buoyant geometry and expressive motion      |
| `high-contrast` | severe geometry and maximum state separation         |
| `headless`      | token boundary without an opinionated visual voice   |

Presets change geometry, type, depth, and motion—not only color. Accessibility
policy remains constant: visible keyboard focus, coarse-pointer targets,
forced-colors mappings, and reduced-motion behavior.

Read the [UI styles guide](../../docs/content/guide/ui-styles.md) for theme
scoping, teleported content, public tokens, and per-component CSS exports.

## Customize without coupling to markup

Components expose semantic state rather than generated class names:

```html
<button
  data-vize-ui="button"
  data-state="idle"
  data-variant="solid"
  data-tone="accent"
  data-size="md"
>
  Save changes
</button>
```

The optional CSS is authored with native nesting and built through Lightning
CSS for the declared browser floor. Consumers receive ordinary CSS and do not
inherit the package's CSS toolchain.

Customize public properties at the nearest product boundary:

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

Application CSS outside Vize's cascade layers wins normally. Tailwind, UnoCSS,
CSS Modules, scoped styles, and plain stylesheets can all consume the same
semantic attributes; none is privileged by the component API.

## Renderers and frameworks

The package's public implementation is real Vue SFC source. The component
contract is checked across DOM, SSR, hydration, and renderer compilation.

| Host        | Integration path                                                                                                                         |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Vue / Vite+ | Import public component subpaths from any SFC.                                                                                           |
| Nuxt        | Import directly or enable `@vizejs/nuxt` UI auto-registration. See the [Nuxt guide](../../docs/content/integrations/nuxt.md).            |
| Void        | Import from an `@void/vue` page; optional CSS remains in the route graph. See the [Void guide](../../docs/content/integrations/void.md). |
| Vapor       | Keep Vue runtime, compiler, and server renderer on the same qualified version. See the [Vapor guide](../../docs/content/guide/vapor.md). |

The standard DOM and SSR builds support Vue 3.5+. Vapor is experimental and
currently requires Vue **3.6.0-rc.11 or later**. The qualified Vapor version is
rc.11; use that version for the runtime, compiler, and server renderer together.

Vue rc.9 throws while hydrating a Vapor root containing CSS custom properties
under a VDOM parent. Later releases also changed slot fallback markup between
SSR and hydration. Mixing Vue 3.5 server output with a newer Vapor client can
therefore produce hydration warnings even when application state agrees.

Some catalogue entries still have tracked Vapor failures. Renderer compatibility
does not certify every legacy component; each generated reference page lists
the gates that its source currently declares.

Maintainers can replay the original rc.9 comparison with:

```bash
VIZE_UI_VAPOR_RUNTIME=reference vp run --filter './npm/ui' check:vapor-runtime
```

## Own the source

When a product needs complete control, pull the whole component unit:

```bash
vize lib pull button
```

The unit keeps its SFC, public types, optional stylesheet, behavior contract,
examples, stories, and tests together. Source distribution is an ownership
choice, not a different component model.

## Documentation as executable evidence

The [component catalogue](https://vizejs.dev/guide/ui/) is generated from the
same metadata, typed SFCs, behavior tables, and colocated examples used by the
package build. Every component page includes:

- a copyable public-import SFC and live browser preview;
- extracted props, events, slots, and exposed members;
- authored usage guidance when the component unit supplies it;
- the normative state-by-input behavior table;
- declared renderer, type, tree-shaking, and bundle gates.

Start with the [Button reference](https://vizejs.dev/guide/ui/button), or
read the [source architecture](./ARCHITECTURE.md) when contributing a component.
