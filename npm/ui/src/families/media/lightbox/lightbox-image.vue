<script setup lang="ts">
import { computed } from "vue";

import ImageContent from "../image/image-content.vue";
import ImageFallback from "../image/image-fallback.vue";
import ImagePlaceholder from "../image/image-placeholder.vue";
import ImageRoot from "../image/image-root.vue";
import type { ImageSource } from "../image/image-types.ts";

const {
  src,
  alt,
  placeholderDelay = 150,
} = defineProps<{
  /** Image source or ordered fallback chain. @default required */
  readonly src: ImageSource;

  /** Native alternative text. @default required */
  readonly alt: string;

  /**
   * Milliseconds before the placeholder renders, avoiding a flash for cached images.
   *
   * @default 150
   */
  readonly placeholderDelay?: number;
}>();

defineSlots<{
  /** Loading placeholder, e.g. a spinner or blurred preview. */
  placeholder?(): unknown;

  /** Fallback after every source failed. */
  fallback?(): unknown;
}>();

// Sources pass through the Image family's media-source policy before rendering.
const rootProps = computed<{ readonly src: ImageSource }>(() => ({ src }));
</script>

<template>
  <ImageRoot v-bind="rootProps" data-lightbox-part="image">
    <ImageContent :alt loading="eager" decoding="async" fetch-priority="high" />
    <ImagePlaceholder :delay="placeholderDelay">
      <slot name="placeholder" />
    </ImagePlaceholder>
    <ImageFallback>
      <slot name="fallback" />
    </ImageFallback>
  </ImageRoot>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
