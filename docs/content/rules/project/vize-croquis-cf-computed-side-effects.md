---
title: "vize:croquis/cf/computed-side-effects"
---

# `vize:croquis/cf/computed-side-effects`

A computed getter writes state or performs another side effect.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

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
import { useDouble } from './use-double';
const { count, doubled, lastCalculated } = useDouble();
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>
</template>

```

## Bad

Evaluating `doubled` writes `lastCalculated`, so reading a computed value also mutates separate state. That couples the side effect to when the lazy getter is read.

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => {
    const next = count.value * 2;
    lastCalculated.value = next;
    return next;
  });
  return { count, doubled, lastCalculated };
}

```

## Good

The getter only returns the derived number. A separate watcher owns the write to `lastCalculated` when `count` changes, including its initial value.

`use-double.ts`

```ts
import { computed, ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => count.value * 2);
  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });
  return { count, doubled, lastCalculated };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
