<script setup lang="ts">
const x = 1;
</script>

<template>
  <p :title="`x: ${x}px`">not reported</p>
  <p
    :title="
      `x: ${x}px`
    "
  >
    reported
  </p>
</template>
