export const imageCropperRendererFixtures = [
  {
    filename: "ImageCropperConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  ImageCropperArea,
  ImageCropperGrid,
  ImageCropperHandle,
  ImageCropperImage,
  ImageCropperRoot,
  ImageCropperViewport,
} from "./families/media/image-cropper/image-cropper.ts";
import type {
  CropArea,
  ImageCropperHandlePosition,
} from "./families/media/image-cropper/image-cropper.ts";

const crop = ref<CropArea>();
const zoom = ref(1);
const rotation = ref(0);
const handles: readonly ImageCropperHandlePosition[] = ["n", "e", "s", "w", "ne", "nw", "se", "sw"];
</script>

<template>
  <ImageCropperRoot v-model="crop" v-model:zoom="zoom" v-model:rotation="rotation" :aspect-ratio="1">
    <template #default="{ ready }">
      <ImageCropperViewport>
        <ImageCropperImage src="/avatar.jpg" alt="Profile photo" cross-origin="anonymous" />
        <ImageCropperArea>
          <ImageCropperGrid />
          <ImageCropperHandle v-for="position in handles" :key="position" :position />
        </ImageCropperArea>
      </ImageCropperViewport>
      <output>{{ ready ? "Ready" : "Loading" }}</output>
    </template>
  </ImageCropperRoot>
</template>
`,
  },
] as const;
