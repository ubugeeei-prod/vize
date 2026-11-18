<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inject: {
    theme: { from: 'theme', default: 'light' },
    api: 'api',
  },
  provide() {
    return { depth: 1 }
  },
  data() { return { x: 1 } },
})
</script>

<template>
  <div>{{ theme }} {{ api }} {{ x }} {{ missingInjected }}</div>
</template>
