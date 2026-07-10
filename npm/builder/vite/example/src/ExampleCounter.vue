<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <div class="counter">
    <p>Count: {{ count }}</p>
    <button @click="count++">+</button>
    <button @click="count--">-</button>
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
