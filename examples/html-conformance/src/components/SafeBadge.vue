<script setup lang="ts">
defineProps<{ label: string }>();
</script>

<template>
  <span class="badge">{{ label }}</span>
</template>
