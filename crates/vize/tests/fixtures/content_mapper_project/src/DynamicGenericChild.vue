<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ choose: [value: T] }>();
</script>

<template>
  <button type="button">Choose {{ value }}</button>
</template>
