<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
function increment() {
  count.value++;
}

const label: number = "not a number";

defineExpose({ count, increment, label });
</script>

<template>
  <div>{{ count }}</div>
</template>
