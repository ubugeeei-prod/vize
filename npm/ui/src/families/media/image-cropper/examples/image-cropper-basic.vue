<!-- Square avatar cropper with a rule-of-thirds grid, corner handles, and controlled zoom and rotation. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  ImageCropper,
  ImageCropperArea,
  ImageCropperGrid,
  ImageCropperHandle,
  ImageCropperImage,
  ImageCropperViewport,
} from "../image-cropper.ts";

const zoom = ref(1);
const rotation = ref(0);
const zoomId = useId();
// The viewport needs a size to fit the image into.
const viewportSize = { inlineSize: "320px", blockSize: "240px" };

function rotate(): void {
  rotation.value = (rotation.value + 90) % 360;
}
</script>

<template>
  <ImageCropper
    v-slot="{ crop }"
    v-model:zoom="zoom"
    v-model:rotation="rotation"
    :aspect-ratio="1"
    :max-zoom="3"
  >
    <ImageCropperViewport :style="viewportSize">
      <ImageCropperImage src="/media/profile-photo.jpg" alt="Profile photo to crop" />
      <ImageCropperArea>
        <ImageCropperGrid />
        <ImageCropperHandle position="nw" />
        <ImageCropperHandle position="ne" />
        <ImageCropperHandle position="sw" />
        <ImageCropperHandle position="se" />
      </ImageCropperArea>
    </ImageCropperViewport>
    <label :for="zoomId">Zoom</label>
    <input :id="zoomId" v-model.number="zoom" type="range" min="1" max="3" step="0.1" />
    <button type="button" @click="rotate">Rotate 90°</button>
    <output>{{
      crop ? `${Math.round(crop.width)} × ${Math.round(crop.height)} px` : "Loading image"
    }}</output>
  </ImageCropper>
</template>
