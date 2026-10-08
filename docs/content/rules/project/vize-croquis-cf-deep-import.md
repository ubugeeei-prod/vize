---
title: "vize:croquis/cf/deep-import"
---

# `vize:croquis/cf/deep-import`

An import chain is deeper than the project allows.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.

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

`App.vue`

```vue
<script setup lang="ts">
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

## Bad

The entry routes a simple value through `level-one`, `level-two`, and `level-three`, creating an unnecessarily deep import chain for a project that wants a shallow public boundary.

`entry.ts`

```ts
export { label } from './level-one';

```

## Good

The entry uses `public-api.ts`, which re-exports the value directly. The consumer keeps the same imported name while the chain becomes shorter.

`entry.ts`

```ts
export { label } from './public-api';

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
