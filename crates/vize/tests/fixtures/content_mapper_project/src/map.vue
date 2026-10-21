<script setup lang="ts">
const cache = new Map<string, number>();
</script>

<template>
  <p>{{ cache.size }}</p>
</template>
