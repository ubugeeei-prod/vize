<script setup lang="ts">
async function label(): Promise<string> {
  const api = await import("./api");
  return api.describe(api.LABEL.length);
}
</script>

<template>
  <button type="button" @click="label">x</button>
</template>
