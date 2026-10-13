<script setup lang="ts">
import { ref } from "vue";
import { useArrayEvery, useArrayFilter } from "@vizejs/composable/use-array";

const items = ref([
  { id: 1, label: "Passport", packed: true },
  { id: 2, label: "Charger", packed: false },
  { id: 3, label: "Umbrella", packed: false },
]);
const remaining = useArrayFilter(items, (item) => !item.packed);
const allPacked = useArrayEvery(items, (item) => item.packed);
</script>

<template>
  <div class="composable-example">
    <p>Check off a packing list and see what is still missing.</p>
    <ul aria-label="Packing list">
      <li v-for="item in items" :key="item.id">
        <label><input v-model="item.packed" type="checkbox" /> {{ item.label }}</label>
      </li>
    </ul>
    <output aria-live="polite">{{
      allPacked
        ? "Everything is packed."
        : `Still to pack: ${remaining.map((item) => item.label).join(", ")}`
    }}</output>
  </div>
</template>
