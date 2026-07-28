<script setup lang="ts">
import Child from "./Child.vue";

const total = 3;
</script>

<template>
  <Child :count="total" />
</template>
