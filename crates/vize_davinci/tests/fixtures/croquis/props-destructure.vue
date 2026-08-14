<script setup lang="ts">
interface Props {
  title: string;
  count?: number;
  tags?: string[];
}

const { title, count = 0, tags = [] } = defineProps<Props>();

console.log(title.toUpperCase(), count.toFixed(0), tags.join(","));
</script>

<template>
  <div>{{ title }} {{ count }} {{ tags.length }}</div>
</template>
