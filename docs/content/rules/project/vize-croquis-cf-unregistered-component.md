---
title: "vize:croquis/cf/unregistered-component"
---

# `vize:croquis/cf/unregistered-component`

A template uses a component that is not registered or imported.

| Default severity | Automatic fix | Options |
| --- | --- | --- |
| error | None; review related files and apply the repair | No per-code options; supported CLI findings accept severity overrides |

Applies to: Analyzed component graph and the supported facts described below

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Bad

A `Child.vue` file exists, but the parent neither imports nor otherwise registers `Child` for its template.

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

## Good

Import `Child` in the parent's script setup so the template resolves the component binding.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

## Configuration (Vite+)

This code is not currently emitted by the CLI, so there is no CLI rule configuration to enable here.

## Related

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Cross-file index](../cross-file.md)
