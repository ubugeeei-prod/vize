<script setup lang="ts" generic="T extends Record<string, unknown>">
import Child from "./Child.vue";

defineProps<{ item: T }>();
</script>

<template>
  <main>
    <p>Intro <div>a baseline finding the seeds must move, not change</div></p>
    <Child />
  </main>
</template>
