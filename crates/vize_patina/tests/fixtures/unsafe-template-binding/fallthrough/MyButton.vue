<script setup lang="ts">
defineProps<{ color?: string }>();
</script>

<template>
  <button type="button" class="my-button"><slot></slot></button>
</template>
