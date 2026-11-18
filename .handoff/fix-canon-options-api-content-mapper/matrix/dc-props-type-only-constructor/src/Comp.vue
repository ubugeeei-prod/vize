<script lang="ts">
import { defineComponent } from 'vue'
class Point { constructor(public x: number, public y: number) {} }
export default defineComponent({
  props: {
    s: String, n: Number, b: Boolean, a: Array, o: Object, f: Function, sym: Symbol, p: Point,
  },
})
</script>

<template>
  <div>{{ s?.length }} {{ n?.toFixed() }} {{ b }} {{ a?.length }} {{ o }} {{ p?.x }} {{ p?.z }}</div>
</template>
