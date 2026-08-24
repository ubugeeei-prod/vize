<script lang="ts">
export default {}
</script>
<script setup lang="ts">
const label = "send"
const emit = defineEmits<{ submit: [value: string] }>()
</script>
<template><button @click="emit('submit', label)">{{ label }}</button></template>
