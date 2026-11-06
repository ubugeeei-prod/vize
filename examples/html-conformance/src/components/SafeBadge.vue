<script setup lang="ts">
const { label } = defineProps<{ label: string }>();
</script>

<template>
  <span class="badge">{{ label }}</span>
</template>
