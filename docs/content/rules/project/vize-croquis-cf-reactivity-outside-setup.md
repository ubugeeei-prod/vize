---
title: "vize:croquis/cf/reactivity-outside-setup"
---

# `vize:croquis/cf/reactivity-outside-setup`

A reactive API is called outside `setup`.

| Default severity | Automatic fix | Options |
| --- | --- | --- |
| Not emitted | None; review related files and apply the repair | No per-code options; supported CLI findings accept severity overrides |

Applies to: Analyzed component graph and the supported facts described below

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.

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
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

## Bad

Both reactive APIs run while the module loads. The two Counter instances therefore share one ref and computed value despite the intended independent counters.

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

## Good

`useCounter` creates the ref and computed synchronously inside each component setup call, giving each widget its own state and tracked derivation.

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

## Configuration (Vite+)

This code is not currently emitted by the CLI, so there is no CLI rule configuration to enable here.

## Related

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
