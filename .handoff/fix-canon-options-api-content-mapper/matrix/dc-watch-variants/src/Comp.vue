<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { a: { b: 1 }, list: [1, 2] } },
  watch: {
    'a.b': { handler: 'onChange', deep: true, immediate: true },
    list: [{ handler(v: number[]) { return v.length } }, 'onChange'],
    a(v: { b: number }) { this.onChange(v.b) },
  },
  methods: { onChange(v: number) { return v } },
})
</script>

<template><div>{{ a.b }} {{ list.length }}</div></template>
