<script setup lang="ts">
defineProps<{ label?: string }>();
</script>
<template><button class="native-host">{{ label ?? 'Native host' }}</button></template>
<style scoped>.native-host { color: rgb(12, 34, 56); }</style>
