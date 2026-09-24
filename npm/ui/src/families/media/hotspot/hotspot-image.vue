<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { normalizeMediaSource } from "../../../media/media-source.ts";
import type { HotspotImageExpose } from "./hotspot-types.ts";

const {
  src,
  alt,
  width = undefined,
  height = undefined,
  loading = "lazy",
} = defineProps<{
  /** Image source. Unsafe or malformed sources are not rendered. @default required */
  readonly src: string;

  /** Alternative text describing the whole image. @default required */
  readonly alt: string;

  /**
   * Intrinsic width reserving layout space.
   *
   * @default undefined
   */
  readonly width?: number | string;

  /**
   * Intrinsic height reserving layout space.
   *
   * @default undefined
   */
  readonly height?: number | string;

  /**
   * Native loading policy.
   *
   * @default "lazy"
   */
  readonly loading?: "eager" | "lazy";
}>();

const element = useTemplateRef<HTMLImageElement>("element");
const safeSrc = computed(() => {
  try {
    return normalizeMediaSource(src, { kind: "image" });
  } catch {
    return undefined;
  }
});
const imageProps = computed<{ readonly src?: string }>(() =>
  safeSrc.value === undefined ? {} : { src: safeSrc.value },
);

const exposed = { element } satisfies {
  readonly element: typeof element;
} & Omit<HotspotImageExpose, "element">;

defineExpose(exposed);
</script>

<template>
  <img
    ref="element"
    v-bind="imageProps"
    :alt
    :width
    :height
    :loading
    draggable="false"
    data-vize-ui="hotspot-image"
    part="image"
    :data-invalid="safeSrc === undefined ? 'true' : undefined"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
