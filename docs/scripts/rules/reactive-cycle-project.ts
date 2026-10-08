import type { ProjectExample } from "./types.ts";
export const reactiveCycleExample: ProjectExample = {
  shared: {
    "main.ts": `import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');`,
    "index.html": `<div id="app"></div>
<script type="module" src="/main.ts"></script>`,
    "count-key.ts": `import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');`,
    "App.vue": `<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>`,
  },
  bad: {
    "CycleView.vue": `<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>`,
  },
  good: {
    "CycleView.vue": `<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>`,
  },
  badExplanation: {
    en: "App owns and provides count (A). CycleView derives nextCount (B), then immediately writes each derived value back into the same injected count. Every write changes the input to the computation again, creating update feedback A → B → A. The identities in the retained graph below represent these two references, not unrelated bindings with matching names.",
    ja: "App が count（A）を保持して provide します。CycleView は nextCount（B）を computed で求め、watch でその値を同じ inject 済みの count に書き戻します。immediate の実行後も計算の入力が更新され続けるため、A → B → A の更新の循環になります。下の参照グラフはこの二つの参照を表し、同名の無関係な変数をつなぐものではありません。",
  },
  goodExplanation: {
    en: "Remove the watcher that writes B back into A. App keeps ownership of count and changes it only through its explicit Increment action; CycleView reads the derived nextCount without feeding the result back. The same references retain only the A → B dependency.",
    ja: "B の値を A に書き戻す watch を取り除きます。count は App が保持し、明示的な Increment 操作でだけ変更します。CycleView は nextCount を読み取るだけで計算結果を書き戻しません。同じ二つの参照には A → B の依存だけが残ります。",
  },
  badGraph:
    "Tracked references: A = provider source; B = consumer reference\nTracked flows: A -> B; B -> A",
  goodGraph:
    "Tracked references: A = provider source; B = consumer reference\nTracked flows: A -> B",
};
