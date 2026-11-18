<script lang="ts">
export default {
  props: { price: { type: Number, required: true } },
  data() { return { label: 'x' } },
  filters: { money(n: number) { return n.toFixed(2) } },
}
</script>

<template>
  <div>{{ price | money }} {{ label.toUpperCase() }} {{ label.toFixed() }}</div>
</template>
