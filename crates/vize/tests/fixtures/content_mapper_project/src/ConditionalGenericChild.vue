<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ confirm: [value: T] }>();
</script>

<template>
  <button type="button">Confirm {{ value }}</button>
</template>
