<script lang="ts">
export default {
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
  },
  emits: ['pick'],
}
</script>

<template><div>{{ title }}{{ size }}</div></template>
