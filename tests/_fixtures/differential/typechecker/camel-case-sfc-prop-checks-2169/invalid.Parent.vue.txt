<script setup lang="ts">
import Child from "./Child.vue";

const wrong: string = "not a number";
</script>

<template>
  <Child :countTotal="wrong" />
</template>
