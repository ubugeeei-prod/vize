<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  expose: ['reset'],
  data() { return { v: 1 } },
  methods: { reset() { this.v = 0 }, hidden() { return 1 } },
})
</script>

<template><div ref="root">{{ v }}</div></template>
