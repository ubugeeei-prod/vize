<script setup lang="ts">
defineProps<{ title?: string }>();
defineEmits<{ close: [] }>();
</script>

<template>
  <section :title="title"><slot /></section>
</template>
