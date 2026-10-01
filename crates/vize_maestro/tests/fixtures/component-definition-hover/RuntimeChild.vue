<script setup lang="ts">
defineProps({ 'font-size': { type: Number, required: true } })
defineEmits(['update:title'])
defineSlots<{ 'header-title'(props: { value: string }): unknown }>()
defineModel<boolean>('quoted"model')
</script>
<template><slot name="header-title" value="ready" /></template>
