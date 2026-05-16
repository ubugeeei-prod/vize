<script setup lang="ts">
const props = defineProps({
  title: String,
  count: {
    type: Number,
    required: true,
  },
  enabled: Boolean,
});

console.log(props.title?.toUpperCase(), props.count.toFixed(0), props.enabled === true);
</script>

<template>
  <div>{{ title }} {{ count }} {{ enabled }}</div>
</template>
