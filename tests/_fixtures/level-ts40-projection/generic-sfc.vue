<script setup lang="ts" generic="T extends { label: string }">
defineProps<{ item: T }>()
</script>
<template><strong>{{ item.label }}</strong></template>
