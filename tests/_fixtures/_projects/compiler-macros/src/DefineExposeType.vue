<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
function increment() {
  count.value++;
}

defineExpose({ count, increment });
</script>

<template>
  <div>{{ count }}</div>
</template>
