<script setup lang="ts">
import Guard from "./Guard.vue";
</script>

<template>
  <Guard
    :check="
      (params) =>
        !!params.id
    "
  >
    ok
  </Guard>
</template>
