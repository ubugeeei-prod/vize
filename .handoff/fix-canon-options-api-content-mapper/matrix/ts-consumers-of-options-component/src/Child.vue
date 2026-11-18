<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { title: { type: String, required: true }, size: { type: Number, default: 1 } },
  emits: { pick: (n: number) => true },
  data() { return { open: false } },
  methods: { toggle(force?: boolean) { this.open = force ?? !this.open; return this.open } },
})
</script>

<template><div>{{ title }}</div></template>
