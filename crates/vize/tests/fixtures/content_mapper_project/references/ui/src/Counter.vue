<script setup lang="ts">
defineProps<{ count: number }>();
</script>

<template>
  <button>{{ count.toFixed(0) }}</button>
</template>
