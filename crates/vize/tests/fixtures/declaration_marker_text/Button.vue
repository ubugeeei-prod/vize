<script setup lang="ts">
export interface PublicProps {
  label: "__vizeComponentMarker"
}
defineProps<PublicProps>()
</script>
<template>{{ label }}</template>
