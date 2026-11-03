<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{ label: string; initial?: number }>();
const emit = defineEmits<{ change: [value: number] }>();

const count = ref(props.initial ?? 0);
const doubled = computed(() => count.value * 2);

function increment() {
  count.value += 1;
  emit("change", count.value);
}
</script>

<template>
  <button class="counter" type="button" @click="increment">
    <span class="counter-label">{{ label }}</span>
    <span class="counter-value">{{ count }} ({{ doubled }})</span>
  </button>
</template>

<style scoped>
.counter {
  display: inline-flex;
  gap: 0.5rem;
}
.counter-value {
  font-variant-numeric: tabular-nums;
}
</style>
