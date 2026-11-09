<script lang="ts">
import { defineComponent, type SlotsType } from 'vue'

export default defineComponent({
  props: {
    /** The title shown above. */
    title: { type: String, required: true },
    count: Number,
  },
  emits: {
    /** Fired on change. */
    change: (value: string) => typeof value === 'string',
  },
  slots: Object as SlotsType<{
    /** Row slot. */
    item: { row: number }
  }>,
})
</script>

<template>
  <div>{{ title }}<slot name="item" :row="count ?? 0" /></div>
</template>
