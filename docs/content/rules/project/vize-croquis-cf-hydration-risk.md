---
title: "vize:croquis/cf/hydration-risk"
---

# `vize:croquis/cf/hydration-risk`

This code groups several reactivity findings, including a non-reactive watch source. It does not imply that every Date.now() expression is detected by the cross-file pass.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Shared project files

Use these unchanged files in both Bad and Good. Install Vue (and vue-router for Router examples) in the project. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
createApp(App).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Bad

`App.vue`

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
watch(count.value, () => {});
</script>
<template><p>{{ count }}</p></template>
```

## Good

`App.vue`

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
watch(() => count.value, () => {});
</script>
<template><p>{{ count }}</p></template>
```

Good avoids this finding; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](../cross-file.md)
