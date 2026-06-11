<script setup lang="ts">
// Auto-registered as <TheGreeting> via `.nuxt/components.d.ts`. The `name`
// prop is required and strongly typed.
defineProps<{
  name: string;
}>();
</script>

<template>
  <p class="greeting">Hello, {{ name }}!</p>
</template>
