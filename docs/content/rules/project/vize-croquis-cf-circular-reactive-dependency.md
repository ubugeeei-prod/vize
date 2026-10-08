---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

Reactive computations depend on each other in a cycle.

Default severity: context-dependent  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Example qualification: `illustrative-source-pair`

The complete Vue project below illustrates update feedback and its repair. It is not a qualified CLI finding witness: the diagnostic producer requires retained reactive-flow reference identities and edges, as shown by the accompanying graph. These sources do not establish that the current source path will emit this exact code. Dedicated tracked-ID graph finding controls remain separate from source grammar checks.

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
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

## Bad

App owns and provides count (A). CycleView derives nextCount (B), then immediately writes each derived value back into the same injected count. Every write changes the input to the computation again, creating update feedback A → B → A. The identities in the retained graph below represent these two references, not unrelated bindings with matching names.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## Good

Remove the watcher that writes B back into A. App keeps ownership of count and changes it only through its explicit Increment action; CycleView reads the derived nextCount without feeding the result back. The same references retain only the A → B dependency.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Cross-file index](../cross-file.md)
