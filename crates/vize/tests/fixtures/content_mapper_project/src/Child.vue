<script setup lang="ts">
defineProps<{ count: number }>();
</script>

<template>
  <output>{{ count.toFixed(0) }}</output>
</template>
