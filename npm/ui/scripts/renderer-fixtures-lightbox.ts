export const lightboxRendererFixtures = [
  {
    filename: "LightboxConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  LightboxClose,
  LightboxContent,
  LightboxCounter,
  LightboxImage,
  LightboxItem,
  LightboxNext,
  LightboxPrevious,
  LightboxRoot,
  LightboxThumbnail,
  LightboxThumbnails,
  LightboxTrigger,
} from "./families/media/lightbox/lightbox.ts";

interface Photo {
  readonly src: string;
  readonly alt: string;
}

const photos: readonly Photo[] = [
  { src: "/photos/harbor.jpg", alt: "Harbor at dusk" },
  { src: "/photos/forest.jpg", alt: "Forest trail" },
];
const open = ref(false);
const index = ref(0);

function preloadSrc(photo: Photo): string {
  return photo.src;
}
</script>

<template>
  <LightboxRoot
    v-model:open="open"
    v-model:index="index"
    :items="photos"
    :get-preload-src="preloadSrc"
    loop
  >
    <template #default="{ item }">
      <LightboxTrigger v-for="(photo, photoIndex) in photos" :key="photo.src" :index="photoIndex">
        <img :src="photo.src" :alt="photo.alt" />
      </LightboxTrigger>
      <LightboxContent>
        <LightboxItem>
          <LightboxImage v-if="item" :src="item.src" :alt="item.alt" />
        </LightboxItem>
        <LightboxPrevious />
        <LightboxNext />
        <LightboxClose />
        <LightboxCounter />
        <LightboxThumbnails>
          <LightboxThumbnail v-for="(photo, photoIndex) in photos" :key="photo.src" :index="photoIndex" />
        </LightboxThumbnails>
      </LightboxContent>
    </template>
  </LightboxRoot>
</template>
`,
  },
] as const;
