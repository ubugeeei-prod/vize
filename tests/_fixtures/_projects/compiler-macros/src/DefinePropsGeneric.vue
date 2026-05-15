<script setup lang="ts" generic="T extends string | number">
const props = defineProps<{
  items: T[];
  selected: T;
}>();
console.log(props.items, props.selected);
</script>

<template>
  <ul>
    <li v-for="item in items" :key="String(item)">{{ item }}</li>
  </ul>
</template>
