<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ select: [value: T] }>();
</script>

<template>
  <button type="button">Select {{ value }}</button>
</template>
