<script setup lang="ts">
import { createStateStore, defineStateModel } from "../src/index.ts";

export const counterModel = defineStateModel({
  key: "counter.panel",
  source: "./CounterPanel.vue",
  version: 1,
  initialState: { count: 0 },
  reducer(state, action: { readonly type: "increment"; readonly by: number }) {
    return { count: state.count + action.by };
  },
});

const store = createStateStore(counterModel);
</script>

<template>
  <output>{{ store.state.count }}</output>
</template>
