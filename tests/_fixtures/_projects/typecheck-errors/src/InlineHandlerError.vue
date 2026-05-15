<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);

function processString(s: string): void {
  console.log(s);
}
</script>

<template>
  <button @click="processString(count)">Click</button>
</template>
