---
title: "vue/use-v-on-exact"
---

# `vue/use-v-on-exact`

Enforce `.exact` modifier on `v-on` when there are modifier-based handlers

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The plain click handler can also run on Ctrl-click, overlapping the separate `.ctrl` handler.

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## Good

`.exact` limits the ordinary click handler to clicks without modifier keys; the Ctrl-specific handler remains separate.

```vue
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

## When handlers overlap

The rule compares listeners for the same event on a native element. It ignores
custom component events unless both listeners use `.native`. Different event
names and disjoint keyboard filters do not conflict.

System modifiers are required keys, so `@keydown.alt.shift` also accepts
Ctrl+Alt+Shift. If a second handler uses `@keydown.alt.ctrl.shift`, add `.exact`
to the broader handler to keep those actions separate. `.exact` permits its
declared system modifiers and rejects additional Ctrl, Shift, Alt or Meta keys.

Multiple keyboard modifiers are alternatives: `@keydown.up.down` accepts either
arrow key. It can overlap `@keydown.ctrl.down` on Ctrl+ArrowDown. The rule reports
the directive key that needs `.exact`, excluding its handler expression.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [All rules](../all.md)
