<script setup lang="ts">
import Child from "../src/Child.vue";
</script>

<template>
  <Child :count="'wrong'" />
</template>
