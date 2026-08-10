<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ activate: [value: T] }>();
</script>

<template>
  <button type="button">Activate {{ value }}</button>
</template>
