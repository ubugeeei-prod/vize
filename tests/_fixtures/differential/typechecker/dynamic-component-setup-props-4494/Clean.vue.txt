<script setup lang="ts">
import Child from "./Child.vue";

const Comp = Child;
</script>

<template>
  <component :is="Comp" :count="1" />
</template>
