<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: ['a', 'b'],
  data() { return { own: 1 } },
})
</script>

<template><div>{{ a.whatever }} {{ b }} {{ own.toFixed() }} {{ c }}</div></template>
