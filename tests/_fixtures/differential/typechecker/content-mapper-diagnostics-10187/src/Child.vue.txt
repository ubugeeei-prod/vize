<script setup lang="ts">
defineProps<{ count: number; label?: string }>();
</script>

<template>
  <div>{{ label }} {{ count }}</div>
</template>
