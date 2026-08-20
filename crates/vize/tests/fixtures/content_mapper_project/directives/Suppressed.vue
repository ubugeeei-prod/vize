<script setup lang="ts">
const count = 1
</script>

<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}
  <!-- @vue-ignore -->
  {{ count.missing }}
</template>
