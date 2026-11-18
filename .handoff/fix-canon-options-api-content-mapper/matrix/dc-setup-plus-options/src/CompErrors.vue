<script lang="ts">
import { defineComponent, ref } from 'vue'
export default defineComponent({
  setup() {
    const n = ref(1)
    return { n, bump: (by: number) => { n.value += by } }
  },
  data() { return { extra: 'e' } },
})
</script>

<template>
  <div>
    {{ n.toUpperCase() }} {{ extra.toFixed() }}
    <button @click="bump('x')">+</button>
  </div>
</template>
