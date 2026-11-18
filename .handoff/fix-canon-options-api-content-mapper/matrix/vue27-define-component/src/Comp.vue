<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    items: { type: Array as PropType<number[]>, default: () => [] },
  },
  data() { return { local: 1 } },
  computed: { upper(): string { return this.title.toUpperCase() } },
  methods: { inc(by: number) { this.local += by; this.$emit('change', by) } },
})
</script>

<template>
  <div>
    {{ title.toUpperCase() }} {{ items.length }} {{ local.toFixed() }} {{ upper.length }}
    <button @click="inc(1)">+</button>
    {{ title.toFixed() }} {{ local.length }} {{ nope }}
    <button @click="inc('x')">x</button>
  </div>
</template>
