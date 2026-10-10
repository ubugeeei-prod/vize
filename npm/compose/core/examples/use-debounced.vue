<script setup lang="ts">
import { computed, ref, useId } from "vue";
import { useDebounced } from "@vizejs/composable/use-debounced";

const query = ref("");
const queryId = useId();
const { debounced, pending, flush, cancel } = useDebounced(query, 500);
const guides = [
  "Getting started",
  "Component styling",
  "Composable reference",
  "Keyboard accessibility",
];
const results = computed(() =>
  guides.filter((guide) => guide.toLowerCase().includes(debounced.value.toLowerCase())),
);
</script>

<template>
  <div class="composable-example">
    <label :for="queryId">Find a guide</label>
    <input :id="queryId" v-model="query" type="search" placeholder="Try component" />
    <output aria-live="polite">{{
      pending ? "Waiting for typing to stop…" : `Showing results for: ${debounced || "all guides"}`
    }}</output>
    <div class="example-actions">
      <button type="button" :disabled="!pending" @click="() => flush()">Search now</button>
      <button type="button" :disabled="!pending" @click="() => cancel()">
        Cancel pending search
      </button>
    </div>
    <ul aria-label="Matching guides">
      <li v-for="guide in results" :key="guide">{{ guide }}</li>
    </ul>
    <p v-if="results.length === 0">No guides match this search.</p>
  </div>
</template>
