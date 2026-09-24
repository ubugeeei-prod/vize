<script setup lang="ts">
import { computed, onMounted, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { imageContext } from "./image-context.ts";
import type {
  ImageContentExpose,
  ImageCrossOrigin,
  ImageDecoding,
  ImageFetchPriority,
  ImageLoading,
  ImageReferrerPolicy,
  ImageSlotState,
  ImageStatus,
} from "./image-types.ts";

const {
  alt,
  srcset = undefined,
  sizes = undefined,
  width = undefined,
  height = undefined,
  loading = "lazy",
  decoding = "async",
  fetchPriority = undefined,
  crossOrigin = undefined,
  referrerPolicy = undefined,
} = defineProps<{
  /** Native alternative text. Pass `""` for decorative images. @default required */
  readonly alt: string;

  /**
   * Responsive candidates for the first source. Dropped once the chain falls back,
   * so a failing `srcset` cannot shadow later candidates.
   *
   * @default undefined
   */
  readonly srcset?: string;

  /**
   * Native `sizes` paired with {@link srcset}.
   *
   * @default undefined
   */
  readonly sizes?: string;

  /**
   * Intrinsic width used by the browser to reserve layout space.
   *
   * @default undefined
   */
  readonly width?: number | string;

  /**
   * Intrinsic height used by the browser to reserve layout space.
   *
   * @default undefined
   */
  readonly height?: number | string;

  /**
   * Native loading policy.
   *
   * @default "lazy"
   */
  readonly loading?: ImageLoading;

  /**
   * Native decoding policy.
   *
   * @default "async"
   */
  readonly decoding?: ImageDecoding;

  /**
   * Native fetch-priority hint.
   *
   * @default undefined
   */
  readonly fetchPriority?: ImageFetchPriority;

  /**
   * Native CORS policy.
   *
   * @default undefined
   */
  readonly crossOrigin?: ImageCrossOrigin;

  /**
   * Native referrer policy.
   *
   * @default undefined
   */
  readonly referrerPolicy?: ImageReferrerPolicy;
}>();

const context = imageContext.use();
const element = useTemplateRef<HTMLImageElement>("element");
const rendered = computed(() => context.status.value !== "error");
const primary = computed(() => context.candidateIndex.value === 0);
const imageProps = computed(() => ({
  crossorigin: crossOrigin,
  decoding,
  fetchpriority: fetchPriority,
  height,
  loading,
  referrerpolicy: referrerPolicy,
  sizes: primary.value ? sizes : undefined,
  src: context.src.value,
  srcset: primary.value ? srcset : undefined,
  width,
}));

/**
 * A server-rendered image can settle before hydration attaches listeners. Read
 * the settled native state once so the lifecycle does not stay in `loading`.
 */
onMounted(() => {
  if (element.value === null || context.status.value !== "loading" || !element.value.complete) {
    return;
  }
  if (element.value.naturalWidth > 0) {
    context.handleLoad(null);
    return;
  }
  if (element.value.getAttribute("src") === null || typeof element.value.decode !== "function") {
    return;
  }
  void settleDecodedImage(element.value, context.src.value);
});

async function settleDecodedImage(image: HTMLImageElement, settledSrc: string | undefined) {
  let decoded = true;
  try {
    await image.decode();
  } catch {
    decoded = false;
  }
  if (context.src.value !== settledSrc) return;
  if (decoded) context.handleLoad(null);
  else context.handleError(null);
}

type ImageContentSetupExpose = Omit<ImageContentExpose, keyof ImageSlotState | "element"> & {
  readonly candidateIndex: ComputedRef<number>;
  readonly element: typeof element;
  readonly src: ComputedRef<string | undefined>;
  readonly status: ComputedRef<ImageStatus>;
  readonly candidateCount: ComputedRef<number>;
};

const exposed = {
  candidateCount: computed(() => context.slotState.value.candidateCount),
  candidateIndex: context.candidateIndex,
  element,
  src: context.src,
  status: context.status,
} satisfies ImageContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <!-- eslint-disable vue/no-root-v-if -->
  <img
    v-if="rendered"
    ref="element"
    :alt
    v-bind="imageProps"
    data-vize-ui="image-content"
    part="content"
    :data-status="context.status.value"
    :data-candidate="context.candidateIndex.value"
    @load="context.handleLoad"
    @error="context.handleError"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
