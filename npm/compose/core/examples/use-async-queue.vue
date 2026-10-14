<script setup lang="ts">
import { computed } from "vue";
import { useAsyncQueue } from "@vizejs/composable/use-async-queue";

const { results, activeIndex, isFinished, run } = useAsyncQueue(
  [
    () => "photo.jpg",
    (file: string) => `${file} resized to 1200 px`,
    (resized: string) => `Uploaded ${resized}`,
  ],
  { immediate: false },
);
const steps = ["Pick a photo", "Resize it", "Upload it"];
const summary = computed(() =>
  isFinished.value ? String(results.value[2]?.data) : `Step ${activeIndex.value + 1} of 3`,
);
</script>

<template>
  <div class="composable-example">
    <p>Run dependent steps in order, passing each result to the next step.</p>
    <button type="button" @click="() => run()">Run steps</button>
    <ol aria-label="Steps">
      <li v-for="(step, index) in steps" :key="step">{{ step }}: {{ results[index]?.state }}</li>
    </ol>
    <output aria-live="polite">{{ summary }}</output>
  </div>
</template>
