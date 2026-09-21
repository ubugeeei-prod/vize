<script setup lang="ts">
defineProps<{ variant?: "primary" | "secondary" }>();
</script>

<template><button /></template>

<art>
  <variant name="Primary" default>
    <Self :variant="123" />
  </variant>
</art>