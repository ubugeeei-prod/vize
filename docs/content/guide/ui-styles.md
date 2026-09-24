---
title: UI styles
---

# UI styles

`@vizejs/ui` components work without a visual stylesheet. To use Vize's
optional styles, import the base, one palette, and the components your page uses:

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-textarea.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-dialog.css";
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

`base.css` supplies semantic tokens, density, and the forced-colors policy; it
is an alias for the existing `theme.css` export. It does not reset application
elements or alter unstyled Vize components. The component files are plain CSS
assets and do not change the JavaScript or Vue API. Importing a component alone
does not add any other component's visual rules.

Input, Textarea, Checkbox, and Switch each have their own CSS export. Text
fields retain native editing and resize behavior; Checkbox distinguishes
checked from mixed, while Switch moves its thumb when the state changes. The
small state transitions stop under `prefers-reduced-motion`. Focus remains
visible for keyboard users, and forced-colors mode keeps native checkbox marks.
The component files reset their local overrides at each theme boundary, so a
Paper form nested inside a Signal page keeps Paper's type and proportions.

The existing ProgressBar and ScrollArea structure and motion recipes are also
published as standalone CSS-only files. Their JavaScript entries already pull
the legacy aggregate stylesheet for required behavior. Import the standalone
file when using the CSS hooks without the JavaScript entry, or when controlling
stylesheets explicitly; avoid importing both paths on one page.

| Preset    | Character                                                 |
| --------- | --------------------------------------------------------- |
| `paper`   | Warm paper, ink, fine borders, and squared controls       |
| `signal`  | Dense graphite surfaces with crisp edges and little depth |
| `atelier` | Quiet studio neutrals with a single restrained accent     |

To offer a style switcher, replace the single preset import above with the
presets you want to offer. Each stylesheet activates only in its own scope:

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

The existing `midnight`, `play`, `high-contrast`, and `headless` presets remain
available. Set `data-vize-theme` on `<html>` when a dialog is teleported to the
document body, so the overlay inherits the same palette as the page. For a
stored preference, `@vizejs/ui/theme-scope` provides a pre-paint bootstrap
script. The old `theme.css`, `theme-preset-*.css`, and `style.css` imports
continue to work; avoid importing `style.css` alongside `base.css` because it
already contains the base and every legacy preset.

Button has short hover, press, and focus feedback. Dialog animates its arrival
without delaying focus or dismissal. It closes immediately because the
headless Dialog currently removes closed content; an exit animation would
require a presence-aware lifecycle. Both visual files disable motion for
`prefers-reduced-motion` and retain clear boundaries in forced-colors mode.
Application CSS outside Vize's cascade layers can override any rule.
