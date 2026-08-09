<script setup lang="ts">
import Child from "./Child.vue";
import { increment } from "./model";

defineProps<{ count: number }>();
</script>

<template>
  <Child :count="increment(count)" />
</template>
