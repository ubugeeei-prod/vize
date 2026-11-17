<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    asChild: { type: Boolean, default: false },
    as: { type: String, default: 'button' },
    label: { type: String, default: '' },
  },
  methods: {
    cx(key: string): string {
      return `p-button-${key}`
    },
  },
})
</script>

<template>
  <button><slot /></button>
</template>
