<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: {
    modelValue: { type: String, required: true },
    maxLength: { type: Number, default: 10 },
    count: { type: Number, default: 0 },
  },
  emits: {
    'update:modelValue': (v: string) => true,
    'update:count': (v: number) => true,
    itemClick: (id: number) => true,
  },
})
</script>

<template><input :value="modelValue" :maxlength="maxLength" /></template>
