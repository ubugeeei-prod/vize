<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);

function increment(): void {
  count.value += 1;
}

function decrement(): void {
  count.value -= 1;
}
</script>

<template>
  <div class="counter">
    <p>Count: {{ count }}</p>
    <button type="button" @click="increment">+</button>
    <button type="button" @click="decrement">-</button>
  </div>
</template>

<style scoped>
.counter {
  margin-top: 1rem;
}

button {
  margin: 0 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 1rem;
  cursor: pointer;
}
</style>
