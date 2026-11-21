<script setup lang="ts">
defineProps<{
  /** Display name. */
  name: string;
  kind: "a" | "b";
  note?: number;
}>();
</script>

<template><div>{{ name }}</div></template>
