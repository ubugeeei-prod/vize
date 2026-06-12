<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <div class="counter">
    <span class="counter__label">Count</span>
    <output class="counter__value">{{ count }}</output>
    <div class="counter__controls">
      <button class="btn" aria-label="Decrement" @click="count--">−</button>
      <button class="btn btn--primary" aria-label="Increment" @click="count++">+</button>
    </div>
  </div>
</template>

<style scoped>
.counter {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 1.75rem 2.25rem;
  background: #ffffff;
  border: 1px solid #e6e8eb;
  border-radius: 18px;
  box-shadow:
    0 1px 2px rgba(16, 24, 40, 0.04),
    0 12px 32px rgba(16, 24, 40, 0.08);
}

.counter__label {
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #8a9099;
}

.counter__value {
  font-size: 3.25rem;
  font-weight: 700;
  line-height: 1;
  font-variant-numeric: tabular-nums;
  color: #1b1f24;
}

.counter__controls {
  display: flex;
  gap: 0.75rem;
}

.btn {
  width: 3rem;
  height: 3rem;
  display: grid;
  place-items: center;
  font-size: 1.5rem;
  font-weight: 600;
  color: #3a4049;
  background: #f3f4f6;
  border: 1px solid #e6e8eb;
  border-radius: 12px;
  cursor: pointer;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    transform 0.06s ease;
}

.btn:hover {
  background: #e9ebef;
}

.btn:active {
  transform: translateY(1px);
}

.btn--primary {
  color: #ffffff;
  background: #42b883;
  border-color: #42b883;
}

.btn--primary:hover {
  background: #369f6e;
  border-color: #369f6e;
}
</style>
