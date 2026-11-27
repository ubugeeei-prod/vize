<script setup lang="ts">
import { computed } from "vue";

import { watermarkImage } from "./watermark-image.ts";
import type { WatermarkProps } from "./types.ts";

const { text = "", opacity = 0.15, gap = 160, rotate = -24 } = defineProps<WatermarkProps>();

defineSlots<{
  /** Content that stays interactive underneath the overlay. */
  default(): unknown;
}>();

const image = computed(() => watermarkImage({ text, gap, rotate }));
const overlayStyle = computed(() => ({
  position: "absolute",
  inset: "0",
  pointerEvents: "none",
  backgroundRepeat: "repeat",
  opacity: String(opacity),
  backgroundImage: `url("${image.value}")`,
}));
</script>

<template>
  <div data-vize-ui="watermark" :style="{ position: 'relative' }">
    <slot />
    <div aria-hidden="true" data-part="overlay" :data-image="image" :style="overlayStyle" />
  </div>
</template>

<style scoped>
/* Headless by design. The overlay is an inert repeated image, not a stylesheet. */
</style>
