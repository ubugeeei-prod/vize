<script setup lang="ts">
const props = defineProps({
  count: {
    type: Number,
    required: true,
  },
  label: String,
});

const asText: string = props.count;
const labelLength = props.label.length;
console.log(asText, labelLength);
</script>

<template>
  <div>{{ count }} {{ label }}</div>
</template>
