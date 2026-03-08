<script setup lang="ts">
import type { SelectBaseProps } from './types'

const props = withDefaults(defineProps<SelectBaseProps>(), {
  size: 'md',
})
</script>

<template>
  <select :class="size" :disabled="disabled ? true : undefined">
    <slot />
  </select>
</template>
