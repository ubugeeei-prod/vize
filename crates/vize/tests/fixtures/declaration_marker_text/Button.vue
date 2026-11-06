<script lang="ts">
export interface PublicProps {
  label: "__vizeComponentMarker"
}
</script>
<script setup lang="ts">
defineProps<PublicProps>()
</script>
<template>{{ label }}</template>
