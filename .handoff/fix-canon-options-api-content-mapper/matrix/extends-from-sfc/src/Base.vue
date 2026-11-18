<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { label: { type: String, required: true } },
  data() { return { baseCount: 0 } },
  computed: { baseUpper(): string { return this.label.toUpperCase() } },
  methods: { baseAdd(n: number) { this.baseCount += n } },
})
</script>

<template><div>{{ label }}</div></template>
