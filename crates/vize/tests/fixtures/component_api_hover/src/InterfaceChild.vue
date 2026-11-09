<script setup lang="ts">
import type { SharedProps } from './types'

interface Props extends SharedProps {
  /** Whether the row is active. */
  active?: boolean
}
const props = defineProps<Props>()

type Emits = {
  /** Fired on change. */
  change: [value: string]
}
const emit = defineEmits<Emits>()

/** The edited name. */
const name = defineModel<string>('name')

defineSlots<{
  /** Row slot. */
  item(props: { /** Row index. */ row: number }): any
}>()

console.log(props.itemCount, name.value)
emit('change', props.title)
</script>

<template>
  <div>{{ title }}<slot name="item" :row="itemCount ?? 0" /></div>
</template>
