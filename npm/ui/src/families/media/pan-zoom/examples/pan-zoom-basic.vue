<!-- Floor plan viewer with pointer, wheel, and keyboard pan and zoom plus zoom, reset, and fit buttons. -->
<script setup lang="ts">
import {
  PanZoom,
  PanZoomContent,
  PanZoomFit,
  PanZoomReset,
  PanZoomStatus,
  PanZoomViewport,
  PanZoomZoomIn,
  PanZoomZoomOut,
} from "../pan-zoom.ts";

// The viewport needs a size to clip and fit its content.
const viewportSize = { inlineSize: "480px", blockSize: "320px" };
</script>

<template>
  <PanZoom :max-scale="4" bounds="contain">
    <PanZoomViewport aria-label="Office floor plan" :style="viewportSize">
      <PanZoomContent>
        <img
          src="https://example.com/media/floor-plan.png"
          alt="Office floor plan"
          width="960"
          height="640"
        />
      </PanZoomContent>
    </PanZoomViewport>
    <PanZoomZoomIn />
    <PanZoomZoomOut />
    <PanZoomReset />
    <PanZoomFit />
    <PanZoomStatus />
  </PanZoom>
</template>
