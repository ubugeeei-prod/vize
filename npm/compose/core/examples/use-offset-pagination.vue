<script setup lang="ts">
import { computed, useId } from "vue";
import { useOffsetPagination } from "@vizejs/composable/use-offset-pagination";

const guides = [
  "Setup",
  "Components",
  "Composables",
  "Styling",
  "Accessibility",
  "Testing",
  "Deployment",
];
const sizeId = useId();
const { currentPage, currentPageSize, pageCount, offset, isFirstPage, isLastPage, prev, next } =
  useOffsetPagination({ total: guides.length, pageSize: 3 });
const visibleGuides = computed(() =>
  guides.slice(offset.value, offset.value + currentPageSize.value),
);
</script>

<template>
  <div class="composable-example">
    <label :for="sizeId">Guides per page</label>
    <select :id="sizeId" v-model.number="currentPageSize">
      <option :value="3">3</option>
      <option :value="5">5</option>
    </select>
    <ul aria-label="Guides on this page">
      <li v-for="guide in visibleGuides" :key="guide">{{ guide }}</li>
    </ul>
    <div class="example-actions">
      <button type="button" :disabled="isFirstPage" @click="prev">Previous page</button>
      <output aria-live="polite">Page {{ currentPage }} of {{ pageCount }}</output>
      <button type="button" :disabled="isLastPage" @click="next">Next page</button>
    </div>
  </div>
</template>
