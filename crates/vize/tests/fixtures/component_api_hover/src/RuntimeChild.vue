<script setup lang="ts">
const props = defineProps({
  /** The title shown above. */
  title: { type: String, required: true },
  count: Number,
})

const emit = defineEmits({
  /** Fired on change. */
  change: (value: string) => typeof value === 'string',
})

console.log(props.title)
emit('change', props.title)
</script>

<template>
  <div>{{ title }}</div>
</template>
