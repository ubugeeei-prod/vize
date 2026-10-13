<script setup lang="ts">
import { computed, ref } from "vue";
import { usePrevious } from "@vizejs/composable/use-previous";

const price = ref(120);
const previous = usePrevious(price);
const change = computed(() => {
  if (previous.value === undefined) return "No earlier price yet.";
  const delta = price.value - previous.value;
  if (delta === 0) return `Unchanged from ${previous.value}.`;
  return `${delta > 0 ? "Up" : "Down"} ${Math.abs(delta)} from ${previous.value}.`;
});
</script>

<template>
  <div class="composable-example">
    <p>Show how a ticket price moved since the last change.</p>
    <label>
      Ticket price
      <input v-model.number="price" type="number" min="0" step="10" />
    </label>
    <output aria-live="polite">{{ change }}</output>
  </div>
</template>
