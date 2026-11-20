<script setup lang="ts">
const payload = await Promise.resolve({
  count: 1,
  label: "ready",
});

const status = `${payload.label}:${payload.count}`;
</script>

<template>
  <div>{{ status }}</div>
</template>
