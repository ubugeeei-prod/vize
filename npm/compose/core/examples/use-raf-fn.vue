<script setup lang="ts">
import { ref } from "vue";
import { useRafFn } from "@vizejs/composable/use-raf-fn";

const progress = ref(0);
const { isActive, pause, resume } = useRafFn(
  ({ delta }) => {
    progress.value = Math.min(100, progress.value + delta / 20);
    if (progress.value >= 100) pause();
  },
  { immediate: false },
);
</script>

<template>
  <div class="composable-example">
    <p>Animate a progress bar frame by frame, independent of the frame rate.</p>
    <progress :value="progress" max="100" aria-label="Upload progress"></progress>
    <div class="example-actions">
      <button type="button" :disabled="isActive || progress >= 100" @click="() => resume()">
        Start
      </button>
      <button type="button" :disabled="!isActive" @click="() => pause()">Pause</button>
      <button type="button" @click="() => (progress = 0)">Reset</button>
    </div>
    <output aria-live="off">{{ Math.round(progress) }}%</output>
  </div>
</template>
