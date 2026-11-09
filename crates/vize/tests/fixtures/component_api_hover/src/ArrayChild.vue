<script setup lang="ts">
const props = defineProps(['title', 'count'])
const emit = defineEmits(['change'])

console.log(props.title)
emit('change', props.title)
</script>

<template>
  <div>{{ title }}</div>
</template>
