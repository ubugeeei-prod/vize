<script setup lang="ts">
const props = defineProps<{
  /** The title shown above. */
  title: string
  /** How many rows. */
  count?: number
}>()

const emit = defineEmits<{
  /** Fired on change. */
  change: [value: string]
  /** The count changed. */
  'update:count': [value: number]
}>()

defineSlots<{
  /** Row slot. */
  item(props: { row: number }): any
}>()

console.log(props.title)
emit('change', props.title)
</script>

<template>
  <div>{{ title }}<slot name="item" :row="count ?? 0" /></div>
</template>
