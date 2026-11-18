<script lang="ts">
import { defineComponent, inject, provide } from 'vue'
import { themeKey } from './keys'
export default defineComponent({
  setup() {
    provide(themeKey, { dark: true })
    const theme = inject(themeKey)!
    return { theme }
  },
  inject: { injected: { from: themeKey as symbol, default: () => ({ dark: false }) } },
})
</script>

<template><div>{{ theme.dark }} {{ theme.light }}</div></template>
