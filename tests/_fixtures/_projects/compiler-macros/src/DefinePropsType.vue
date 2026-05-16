<script setup lang="ts">
const props = defineProps<{
  title: string;
  count: number;
}>();
console.log(props.title, props.count);
</script>

<template>
  <div>{{ title }} - {{ count }}</div>
</template>
