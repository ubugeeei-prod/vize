<!-- Photo gallery whose thumbnails open a modal lightbox with previous/next, counter, and thumbnail strip. -->
<script setup lang="ts">
import {
  Lightbox,
  LightboxClose,
  LightboxContent,
  LightboxCounter,
  LightboxImage,
  LightboxItem,
  LightboxNext,
  LightboxPrevious,
  LightboxThumbnail,
  LightboxThumbnails,
  LightboxTrigger,
} from "../lightbox.ts";

interface Photo {
  readonly src: string;
  readonly alt: string;
}

const photos: readonly Photo[] = [
  { src: "/photos/harbor.jpg", alt: "Fishing boats moored in the harbor at dusk" },
  { src: "/photos/market.jpg", alt: "Fruit stalls at the morning market" },
  { src: "/photos/lighthouse.jpg", alt: "White lighthouse on a rocky cape" },
];

/** Only same-origin paths reach the DOM. */
function safeUrl(url: string): string {
  return /^\/(?!\/)/u.test(url) ? url : "";
}
</script>

<template>
  <Lightbox v-slot="{ item }" :items="photos" loop>
    <LightboxTrigger v-for="(photo, index) in photos" :key="photo.src" :index>
      <img :src="safeUrl(photo.src)" :alt="photo.alt" width="160" height="120" />
    </LightboxTrigger>
    <LightboxContent>
      <LightboxItem>
        <LightboxImage v-if="item" :src="safeUrl(item.src)" :alt="item.alt" />
      </LightboxItem>
      <LightboxPrevious>Previous</LightboxPrevious>
      <LightboxCounter />
      <LightboxNext>Next</LightboxNext>
      <LightboxClose>Close</LightboxClose>
      <LightboxThumbnails>
        <LightboxThumbnail v-for="(photo, index) in photos" :key="photo.src" :index>
          <img :src="safeUrl(photo.src)" alt="" width="48" height="36" />
        </LightboxThumbnail>
      </LightboxThumbnails>
    </LightboxContent>
  </Lightbox>
</template>
