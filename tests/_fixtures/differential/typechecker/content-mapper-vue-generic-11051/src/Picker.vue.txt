<script setup lang="ts" generic="T extends string">
defineProps<{ value: T }>();
</script>

<template>
  <span>{{ value }}</span>
</template>
