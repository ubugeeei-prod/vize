<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: ['changed', 'update:value'],
  props: { value: String },
})
</script>

<template><div /></template>
