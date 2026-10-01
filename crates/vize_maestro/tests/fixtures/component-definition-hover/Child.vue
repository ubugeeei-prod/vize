<script setup lang="ts">
defineProps<{ message: string; count?: number }>()
defineEmits<{ save: [value: number] }>()
defineSlots<{ item(props: { row: string; index: number }): any; empty(): any }>()
defineModel<string>('title')
</script>
<template><div><slot name="item" :row="message" :index="count" /></div></template>
