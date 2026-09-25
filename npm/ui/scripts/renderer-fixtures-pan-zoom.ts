export const panZoomRendererFixtures = [
  {
    filename: "PanZoomConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  PanZoomContent,
  PanZoomFit,
  PanZoomReset,
  PanZoomRoot,
  PanZoomStatus,
  PanZoomViewport,
  PanZoomZoomIn,
  PanZoomZoomOut,
} from "./families/media/pan-zoom/pan-zoom.ts";
import type { PanZoomTransform } from "./families/media/pan-zoom/pan-zoom.ts";

const transform = ref<PanZoomTransform>({ x: 0, y: 0, scale: 1 });
</script>

<template>
  <PanZoomRoot v-model="transform" bounds="contain" :max-scale="6" wheel-mode="zoom-with-ctrl">
    <template #default="{ scale }">
      <PanZoomViewport aria-label="Floor plan">
        <PanZoomContent>
          <img alt="Floor plan" src="/plan.png" width="800" height="600" />
        </PanZoomContent>
      </PanZoomViewport>
      <PanZoomZoomIn />
      <PanZoomZoomOut />
      <PanZoomReset />
      <PanZoomFit />
      <PanZoomStatus />
      <output>{{ Math.round(scale * 100) }}%</output>
    </template>
  </PanZoomRoot>
</template>
`,
  },
] as const;
