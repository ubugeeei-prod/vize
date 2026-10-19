<script setup lang="ts">
import { ref } from "vue";
import type { Props } from "./index.server.ts";

const props = defineProps<Props>();
const clicks = ref(props.count);
</script>

<template>
  <h1 class="title">{{ title }}</h1>
  <button data-test="counter" @click="clicks++">Clicks: {{ clicks }}</button>
</template>

<style scoped>
.title {
  color: rebeccapurple;
}
</style>
