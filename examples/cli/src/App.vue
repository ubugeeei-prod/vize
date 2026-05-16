<script setup lang="ts">
import { ref, computed } from "vue";

const count = ref(0);
const doubled = computed(() => count.value * 2);

function increment() {
  count.value++;
}

function decrement() {
  count.value--;
}
</script>

<template>
  <div class="container">
    <h1>{{ count }}</h1>
    <p>Doubled: {{ doubled }}</p>
    <div class="buttons">
      <button @click="decrement">-1</button>
      <button @click="increment">+1</button>
    </div>
  </div>
</template>

<style scoped>
.container {
  padding: 20px;
  background: #f0f0f0;
}

h1 {
  color: #333;
  font-size: 2rem;
}

.buttons {
  display: flex;
  gap: 10px;
}

button {
  padding: 8px 16px;
  cursor: pointer;
}
</style>
