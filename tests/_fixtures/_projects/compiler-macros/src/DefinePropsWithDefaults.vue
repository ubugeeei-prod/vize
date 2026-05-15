<script setup lang="ts">
interface Props {
  title?: string;
  count?: number;
  items?: string[];
}

const props = withDefaults(defineProps<Props>(), {
  title: "default",
  count: 0,
  items: () => [],
});
console.log(props.title, props.count, props.items);
</script>

<template>
  <div>{{ title }} - {{ count }}</div>
</template>
