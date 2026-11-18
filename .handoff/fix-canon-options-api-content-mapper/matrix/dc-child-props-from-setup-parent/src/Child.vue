<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
    kind: { type: String as PropType<'a' | 'b'>, default: 'a' },
  },
  emits: { pick: (n: number) => true },
})
</script>

<template><div>{{ title }}{{ size }}{{ kind }}</div></template>
