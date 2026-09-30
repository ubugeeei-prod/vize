<script setup lang="ts">
const emit = defineEmits<{
  change: [value: string];
  update: [id: number, value: string];
}>();

function handleClick() {
  emit("change", "hello");
  emit("update", 1, "world");
}
</script>

<template>
  <button @click="handleClick">Click</button>
</template>
