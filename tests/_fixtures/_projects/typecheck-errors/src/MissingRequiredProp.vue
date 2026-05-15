<script setup lang="ts">
const props = defineProps<{
  title: string;
  count: number;
}>();
const value = props.nonExistent;
console.log(value);
</script>

<template>
  <div>{{ props.title }}</div>
</template>
