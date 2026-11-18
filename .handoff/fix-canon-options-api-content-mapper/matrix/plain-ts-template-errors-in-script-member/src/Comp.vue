<script lang="ts">
export default {
  props: { a: { type: String, required: true } },
  data() { return { b: 1 } },
  computed: { c(): string { return this.a + this.b } },
  methods: { m(x: number) { return x + this.b } },
}
</script>

<template>
  <div>{{ a.toUpperCase() }} {{ c.length }} {{ m(1) }} {{ m('x') }} {{ b.length }}</div>
</template>
