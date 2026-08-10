<script setup lang="ts">
defineProps<{ count: number }>();
defineEmits<{
  save: [value: number];
  saveItem: [id: string];
}>();
</script>

<template>
  <output>{{ count.toFixed(0) }}</output>
</template>
