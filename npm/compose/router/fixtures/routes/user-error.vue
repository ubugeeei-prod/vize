<script setup lang="ts">
defineProps<{ message: string }>();
</script>

<template>
  <p role="alert">{{ message }}</p>
</template>
