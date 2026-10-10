<script setup lang="ts">
import { useCounter } from "@vizejs/composable/use-counter";

const { count, atMin, atMax, increment, decrement, reset } = useCounter(1, { min: 1, max: 5 });
</script>

<template>
  <div class="composable-example">
    <p>Reserve up to five workshop seats.</p>
    <div class="example-actions">
      <button type="button" :disabled="atMin" @click="decrement()">Remove seat</button>
      <output aria-live="polite">{{ count }} {{ count === 1 ? "seat" : "seats" }}</output>
      <button type="button" :disabled="atMax" @click="increment()">Add seat</button>
      <button type="button" @click="reset()">Reset seats</button>
    </div>
    <p v-if="atMax">You reached the five-seat limit.</p>
  </div>
</template>
