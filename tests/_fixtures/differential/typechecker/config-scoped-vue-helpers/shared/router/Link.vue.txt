<script setup lang="ts">
defineProps<{ to: string }>();
</script>

<template>
  <a :href="to"><slot /></a>
</template>
