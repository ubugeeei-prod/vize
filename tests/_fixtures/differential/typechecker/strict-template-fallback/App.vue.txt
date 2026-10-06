<script setup lang="ts">
import Child from "./Child.vue";
</script>

<template>
  <!-- plant: extra is not a declared prop -->
  <!-- @plant-error -->
  <Child title="ok" extra="nope" />
</template>
