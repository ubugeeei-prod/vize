<script setup lang="ts">
defineProps<{ check: (params: Record<string, string>) => boolean }>();
</script>

<template>
  <div><slot /></div>
</template>
