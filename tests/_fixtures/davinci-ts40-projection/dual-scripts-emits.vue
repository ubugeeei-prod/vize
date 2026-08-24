<script lang="ts">
export const label = "send"
</script>
<script setup lang="ts">
const emit = defineEmits<{ submit: [value: string] }>()
</script>
<template><button @click="emit('submit', label)">{{ label }}</button></template>
