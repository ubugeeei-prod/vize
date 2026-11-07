---
title: UI styles
description: Add an optional visual voice to @vizejs/ui with native CSS, semantic selectors, and public design tokens.
---

# UI styles

`@vizejs/ui` separates accessible behavior from visual design. Start without
CSS, add one of the shipped voices, or style the same semantic attributes from
your own system. Tailwind is neither required nor privileged.

<iframe src="/component-previews/app/index.html?family=button" title="Compare every Button visual voice and interaction state" loading="lazy" width="100%" height="560" style="border:1px solid #8885;border-radius:12px"></iframe>

[Open the full-screen style lab](/component-previews/app/index.html?family=button)

## Start with three imports

Load the shared tokens, one preset, and only the component styles this screen
uses:

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
```

```vue
<script setup lang="ts">
import { Button } from "@vizejs/ui/button";
</script>

<template>
  <main data-vize-theme="paper">
    <Button type="button" variant="solid" tone="accent">Save changes</Button>
    <Button type="button" variant="outline" tone="neutral">Preview</Button>
    <Button type="button" variant="quiet" tone="danger">Delete</Button>
  </main>
</template>
```

Put the CSS imports in the application entry when every route shares them. A
component import never injects a theme or another component's visual rules.

## Pick a voice, not just a palette

Every preset has distinct geometry, typography, depth, density, and motion.
They all consume the same typed props and semantic DOM contract.

| Preset          | Direction                                                    | Good starting point for              |
| --------------- | ------------------------------------------------------------ | ------------------------------------ |
| `atelier`       | quiet studio neutrals, soft geometry, restrained depth       | productivity and editorial tools    |
| `paper`         | warm ink, fine rules, squared controls, almost no lift       | publishing and information-dense UI |
| `signal`        | compact graphite, crisp edges, monospace detail, little depth | operations and developer tools      |
| `midnight`      | luminous dark surfaces and controlled layered shadows        | media and focused workspaces        |
| `play`          | pill geometry, buoyant depth, expressive press feedback      | social and consumer products        |
| `high-contrast` | severe geometry, heavy separation, no decorative depth       | maximum state legibility            |
| `headless`      | semantic token boundary without an opinionated visual voice  | a fully custom design system        |

Replace the preset import to commit to one voice, or import several when the
product lets people switch:

```ts
import "@vizejs/ui/theme-preset-atelier.css";
import "@vizejs/ui/theme-preset-midnight.css";
import "@vizejs/ui/theme-preset-paper.css";
```

```ts
document.documentElement.dataset.vizeTheme = "midnight";
```

Each stylesheet activates only inside its matching `data-vize-theme` scope.
Nested scopes are supported, including Shadow DOM hosts:

```vue
<template>
  <main data-vize-theme="atelier">
    <EditorWorkspace />

    <aside data-vize-theme="signal">
      <OperationsPanel />
    </aside>
  </main>
</template>
```

Set the theme on `<html>` when Dialog, Tooltip, or another overlay teleports to
`document.body`. Use `@vizejs/ui/theme-scope` when a stored preference must be
applied before first paint.

## Customize through public properties

Visual CSS starts from `data-vize-ui`, then nests typed state beneath it. There
are no BEM classes or generated utility strings to mirror:

```css
.checkout-actions {
  --vize-ui-button-radius: 0.5rem;
  --vize-ui-button-shadow: none;
  --vize-ui-button-shadow-hover: none;
  --vize-ui-button-hover-lift: 0px;
  --vize-ui-button-press-depth: 1px;

  [data-vize-ui="button"] {
    letter-spacing: 0.01em;

    &[data-tone="danger"] {
      --vize-ui-button-fill: var(--checkout-danger);
    }
  }
}
```

Button's public component properties are:

| Property                          | Purpose                                  |
| --------------------------------- | ---------------------------------------- |
| `--vize-ui-button-fill`           | semantic fill and outline source         |
| `--vize-ui-button-ink`            | foreground on the fill                   |
| `--vize-ui-button-radius`         | control geometry                         |
| `--vize-ui-button-shadow`         | resting depth                            |
| `--vize-ui-button-shadow-hover`   | hover depth                              |
| `--vize-ui-button-hover-lift`     | hover translation                        |
| `--vize-ui-button-press-depth`    | pressed translation                      |
| `--vize-ui-button-press-scale`    | pressed scale                            |
| `--vize-ui-button-hover-strength` | filled hover color mix                   |
| `--vize-ui-button-tint-strength`  | outline and quiet hover tint             |

Application CSS outside Vize's cascade layers wins normally. CSS Modules,
scoped styles, Tailwind, UnoCSS, and plain global styles can all consume this
contract without changing the component.

The package sources use native CSS nesting. Lightning CSS lowers that source to
the declared browser floor during the package build, so applications receive
ordinary CSS and do not need the Vize toolchain.

## Keep accessibility policy intact

The `vize.policy` layer follows visual presets and protects interaction
requirements:

- keyboard focus stays visible with `:focus-visible`;
- coarse pointers receive a minimum 44 CSS-pixel target;
- `prefers-reduced-motion` removes decorative movement and the loading spin;
- forced-colors mode maps surfaces, borders, text, focus, and disabled state to
  system colors;
- hover-only feedback runs only when the device actually supports hover.

Override policy only when the replacement remains equivalent. Color contrast,
an accessible name, and honest loading text remain application responsibilities.

## Load only what the route uses

Common visual assets are independently exported:

```ts
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-tabs.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
```

Input, Textarea, Checkbox, Switch, Button, Dialog, Card, Badge, Alert, Tooltip,
Tabs, Breadcrumb, Pagination, and Stepper each have a component-scoped asset.
ProgressBar and ScrollArea also expose CSS-only assets for explicit low-level
use. Avoid importing a standalone asset and a legacy aggregate entry that
already includes the same styles.

`style.css` remains a compatibility aggregate containing the base and legacy
presets. New code should prefer explicit imports; do not combine `style.css`
with `base.css`.

## Framework and renderer notes

The visual layer is static CSS: it does not run setup code, read the browser,
or change server markup.

- In Nuxt, add the CSS exports to `css` in `nuxt.config.ts`, or import them from
  the application entry. See [Nuxt](../integrations/nuxt.md).
- In Void, import them from the page or its application entry so Void includes
  them in the route CSS graph. See [Void](../integrations/void.md).
- DOM, SSR, hydration, and Vapor consume the same semantic attributes. Vapor
  does not require a different stylesheet. See [Vapor](./vapor.md).

Continue with the [component catalogue](./ui/index.md). Each reference page
shows a live example, copyable SFC, typed API, authored guidance, normative
behavior table, and declared quality gates.
