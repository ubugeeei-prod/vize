<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCapturePhotoExpose, WebcamCapturePhotoResult } from "./webcam-capture-types.ts";

const { alt = undefined } = defineProps<{
  /**
   * Alternative text. Defaults to `messages.photo`.
   *
   * @default undefined
   */
  readonly alt?: string;
}>();

defineSlots<{
  /** Rendered instead of nothing while no photo exists, e.g. a placeholder. */
  empty?(props: Record<string, never>): unknown;

  /** Extra content after the image, e.g. a download link. Receives the photo and its URL. */
  default?(props: { readonly photo: WebcamCapturePhotoResult; readonly url: string }): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLImageElement>("element");
const url = computed(() => context.photoUrl.value);
const photo = computed(() => context.photo.value);
const altText = computed(() => alt ?? context.messages.value.photo);
// Only root-created `blob:` object URLs ever reach the image.
const imageAttributes = computed(() => ({
  height: photo.value?.height,
  src: url.value,
  width: photo.value?.width,
}));

type SetupExpose = Omit<WebcamCapturePhotoExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <figure
    data-vize-ui="webcam-capture-photo"
    part="photo"
    :data-state="photo === null ? 'empty' : 'captured'"
  >
    <template v-if="photo !== null && url !== undefined">
      <img
        ref="element"
        :alt="altText"
        v-bind="imageAttributes"
        data-vize-ui="webcam-capture-photo-image"
      />
      <slot :photo :url />
    </template>
    <slot v-else name="empty" />
  </figure>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
