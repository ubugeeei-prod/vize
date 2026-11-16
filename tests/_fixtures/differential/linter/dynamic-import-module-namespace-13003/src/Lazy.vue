<script setup lang="ts">
import Child from "./Child.vue";

async function label(): Promise<string> {
  const api = await import("./api");
  return api.describe(api.LABEL.length);
}
</script>

<template>
  <Child />
  <button type="button" @click="label">x</button>
</template>
