<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ pick: [value: T] }>();
</script>

<template>
  <button type="button">Pick {{ value }}</button>
</template>
