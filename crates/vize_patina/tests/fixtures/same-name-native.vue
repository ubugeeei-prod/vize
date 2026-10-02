<script setup lang="ts">
import { computed } from "vue";
import ChildList from "./ChildList.vue";

const items = ["a", "b"];
const style = computed(() => ({ color: "red" }));
const ariaLabel = "box";
</script>

<template>
  <ChildList :items="items" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
