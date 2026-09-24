<script setup lang="ts">
import { computed, onMounted, useTemplateRef } from "vue";

import { normalizeMediaSource } from "../../../media/media-source.ts";
import { imageCropperContext } from "./image-cropper-context.ts";
import { toViewport } from "./image-cropper-geometry.ts";
import type { ImageCropperPartExpose } from "./image-cropper-types.ts";

const {
  src,
  alt,
  crossOrigin = undefined,
  referrerPolicy = undefined,
  allowInsecure = false,
} = defineProps<{
  /**
   * Image source. Relative, `https:`, `blob:`, and base64 `data:image/*` URLs are
   * accepted; unsafe schemes are dropped. Use CORS-enabled URLs to export crops. @default required
   */
  readonly src: string;

  /**
   * Permit unencrypted `http:` sources for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;

  /** Native alternative text. @default required */
  readonly alt: string;

  /**
   * Native CORS mode; `anonymous` keeps canvas exports untainted for CORS images.
   *
   * @default undefined
   */
  readonly crossOrigin?: "" | "anonymous" | "use-credentials";

  /**
   * Native referrer policy.
   *
   * @default undefined
   */
  readonly referrerPolicy?: ReferrerPolicy;
}>();

const emit = defineEmits<{
  /** Fired after the image loads and its natural size is known. */
  load: [nativeEvent: Event | null];

  /** Fired when the image fails to load. */
  error: [nativeEvent: Event];
}>();

const context = imageCropperContext.use();
const element = useTemplateRef<HTMLImageElement>("element");

const imageAttributes = computed(() => ({
  crossorigin: crossOrigin,
  referrerpolicy: referrerPolicy,
  src: safeSource(src),
}));

function safeSource(source: string): string | undefined {
  try {
    return normalizeMediaSource(source, { kind: "image", allowInsecure });
  } catch {
    return undefined;
  }
}

function px(value: number): string {
  return `${Math.round(value * 100) / 100}px`;
}

// Geometry is applied only once measured, so server and hydration markup agree.
const imageStyle = computed(() => {
  const natural = context.naturalSize.value;
  const bounds = context.bounds.value;
  if (!context.ready.value || natural === null || bounds === null) return undefined;
  const scale = context.scale.value;
  const middle = toViewport(
    { x: bounds.width / 2, y: bounds.height / 2 },
    context.center.value,
    context.viewportSize.value,
    scale,
  );
  const width = natural.width * scale;
  const height = natural.height * scale;
  return {
    height: px(height),
    left: px(middle.x - width / 2),
    maxWidth: "none",
    pointerEvents: "none",
    position: "absolute",
    top: px(middle.y - height / 2),
    transform: `rotate(${context.rotation.value}deg)`,
    transformOrigin: "center",
    userSelect: "none",
    width: px(width),
  } as const;
});

function readSize(nativeEvent: Event | null): void {
  if (element.value === null || element.value.naturalWidth <= 0) return;
  context.setNaturalSize({
    width: element.value.naturalWidth,
    height: element.value.naturalHeight,
  });
  emit("load", nativeEvent);
}

function onLoad(event: Event): void {
  readSize(event);
}

function onError(event: Event): void {
  emit("error", event);
}

onMounted(() => {
  if (element.value?.complete === true) readSize(null);
});

type ImageCropperImageSetupExpose = Omit<ImageCropperPartExpose<HTMLImageElement>, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ImageCropperImageSetupExpose;

defineExpose(exposed);
</script>

<template>
  <img
    ref="element"
    :alt
    v-bind="imageAttributes"
    draggable="false"
    data-vize-ui="image-cropper-image"
    part="image"
    :style="imageStyle"
    @load="onLoad"
    @error="onError"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
