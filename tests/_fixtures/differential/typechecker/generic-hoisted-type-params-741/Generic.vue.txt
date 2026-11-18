<script setup lang="ts" generic="T extends string">
type Option = { key: T; label: string }

defineProps<{
  options: Option[]
  current: T | undefined
}>()
</script>

<template>
  <ul>
    <li v-for="o in options" :key="o.key">{{ o.label }}</li>
  </ul>
</template>
