<script setup lang="ts">
const props = withDefaults(defineProps<{
  /** The title shown above. */
  title?: string
  count?: number
}>(), { title: 'none', count: 0 })

const emit = defineEmits<{
  /** Fired on change. */
  (e: 'change', value: string): void
}>()

console.log(props.title)
emit('change', props.title)
</script>

<template>
  <div>{{ title }}</div>
</template>
