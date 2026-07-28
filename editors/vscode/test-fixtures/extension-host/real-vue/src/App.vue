<script setup lang="ts">
import Child from "./Child.vue";

const label = "hello from vize";
const amount = 41;
</script>

<template>
  <main>
    <Child :count="label" />
    <p>{{ label }}</p>
    <p>{{ amount }}</p>
  </main>
</template>
