<script setup lang="tsx">
import { ref } from "vue";

const count = ref(0);

const CounterPreview = ({ label }: { label: string }) => (
  <section class="counter-preview">
    <p>
      {label}: {count.value}
    </p>
    <button type="button" onClick={() => (count.value += 1)}>
      Increment
    </button>
  </section>
);
</script>

<template>
  <component :is="CounterPreview" label="Preview" />
</template>
