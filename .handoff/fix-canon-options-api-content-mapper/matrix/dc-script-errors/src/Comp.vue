<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { title: { type: String, required: true } },
  emits: ['change'],
  data() { return { n: 1 } },
  methods: {
    run() {
      this.nope()
      this.title = 'x'
      this.$emit('unknown')
      const s: string = this.n
      return s
    },
  },
})
</script>

<template>
  <div>{{ title }}</div>
</template>
