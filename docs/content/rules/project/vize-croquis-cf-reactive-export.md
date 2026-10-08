---
title: "vize:croquis/cf/reactive-export"
---

# `vize:croquis/cf/reactive-export`

Reactive state is exported from the module.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

## Bad

The module exports one initialized reactive object, so every importer receives the same count. In an SSR module shared between requests, this defeats the example’s per-instance/request state isolation.

`state.ts`

```ts
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

## Good

The module exports a factory, and App invokes it inside setup. Each instance obtains a fresh reactive count rather than the exported singleton.

`state.ts`

```ts
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
