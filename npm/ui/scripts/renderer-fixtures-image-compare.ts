export const imageCompareRendererFixtures = [
  {
    filename: "ImageCompareConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  ImageCompareAfter,
  ImageCompareBefore,
  ImageCompareHandle,
  ImageCompareLabel,
  ImageCompareRoot,
} from "./families/media/image-compare/image-compare.ts";

const position = ref(40);

function formatValue(value: number): string {
  return value + "% before";
}
</script>

<template>
  <ImageCompareRoot v-model="position" :step="5" :messages="{ valueText: formatValue }">
    <template #default="{ state }">
      <ImageCompareBefore><img src="/before.jpg" alt="Before" /></ImageCompareBefore>
      <ImageCompareAfter><img src="/after.jpg" alt="After" /></ImageCompareAfter>
      <ImageCompareLabel side="before">Before</ImageCompareLabel>
      <ImageCompareLabel side="after">After {{ state }}</ImageCompareLabel>
      <ImageCompareHandle aria-label="Reveal the retouched photo" />
    </template>
  </ImageCompareRoot>
</template>
`,
  },
] as const;
