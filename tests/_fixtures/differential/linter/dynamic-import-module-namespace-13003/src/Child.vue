<script setup lang="ts">
const { label = "" } = defineProps<{ label?: string }>();
</script>

<template>
  <span>{{ label }}</span>
</template>
