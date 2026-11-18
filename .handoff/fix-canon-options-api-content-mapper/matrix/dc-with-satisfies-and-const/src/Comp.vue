<script lang="ts">
import { defineComponent } from 'vue'
const Comp = defineComponent({
  props: { a: { type: String, required: true } },
  data() { return { b: 1 } },
})
export default Comp
</script>

<template><div>{{ a.toUpperCase() }} {{ b.toFixed() }} {{ a.toFixed() }} {{ c }}</div></template>
