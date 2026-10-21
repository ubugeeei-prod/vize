<script setup lang="ts">
const n = 1;
</script>

<template>
  <div>
    <NotImported />
    <p v-not-registered>{{ n }}</p>
  </div>
</template>
