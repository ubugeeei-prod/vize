<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

import { Text } from "./Text.js";
import { spinnerTypes, type SpinnerProps } from "./display-primitive-types.js";

const {
  type = "dots",
  interval = 80,
  frames: customFrames,
  label,
  fg,
} = defineProps<SpinnerProps>();

defineOptions({ name: "Spinner" });

const frameIndex = ref(0);
const frames = computed(() => {
  const selectedFrames = customFrames ?? spinnerTypes[type] ?? spinnerTypes.dots;
  return selectedFrames.length > 0 ? selectedFrames : spinnerTypes.dots;
});
const content = computed(() => {
  const frame = frames.value[frameIndex.value % frames.value.length];
  return label ? `${frame} ${label}` : frame;
});

let timer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  timer = setInterval(() => {
    frameIndex.value = (frameIndex.value + 1) % frames.value.length;
  }, interval);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<template>
  <Text :content :fg />
</template>
