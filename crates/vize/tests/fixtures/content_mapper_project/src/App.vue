<script setup lang="ts">
import Child from "./Child.vue";
import { increment } from "./model";

defineProps<{ count: number }>();
const handleSaveItem = (id: string) => id;
</script>

<template>
  <Child :count="increment(count)" @save="increment" @save-item="handleSaveItem" />
</template>
