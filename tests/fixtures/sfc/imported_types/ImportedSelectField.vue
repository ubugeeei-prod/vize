<script setup lang="ts">
import type { SelectBaseProps } from './types'

interface SelectFieldProps extends SelectBaseProps {
  items: { label: string; value: string }[]
}

const model = defineModel<string | undefined>({ default: undefined })

withDefaults(defineProps<SelectFieldProps>(), {
  size: 'md',
})
</script>

<template>
  <select v-model="model" :class="size" :disabled="disabled">
    <option v-for="item in items" :key="item.value" :value="item.value">
      {{ item.label }}
    </option>
  </select>
</template>
