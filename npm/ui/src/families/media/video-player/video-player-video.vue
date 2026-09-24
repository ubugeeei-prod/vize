<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { normalizeMediaSource } from "../../../media/media-source.ts";
import { mediaPlayerContext } from "../media-player/media-player-context.ts";
import type {
  MediaPlayerCrossOrigin,
  MediaPlayerMediaExpose,
  MediaPlayerPreload,
  MediaPlayerSlotState,
} from "../media-player/media-player-types.ts";

const {
  src = undefined,
  poster = undefined,
  preload = "metadata",
  playsinline = true,
  autoplay = false,
  loop = false,
  controls = false,
  crossOrigin = undefined,
  disablePictureInPicture = false,
  width = undefined,
  height = undefined,
  allowInsecure = false,
} = defineProps<{
  /**
   * Video source, validated by the shared media-source policy. Unsafe sources are
   * dropped (`data-invalid-src`); use `<source>` children for format fallbacks.
   *
   * @default undefined
   */
  readonly src?: string;

  /**
   * Poster image, validated as an image source.
   *
   * @default undefined
   */
  readonly poster?: string;

  /**
   * Native preload hint.
   *
   * @default "metadata"
   */
  readonly preload?: MediaPlayerPreload;

  /**
   * Play inline on iOS instead of entering native fullscreen.
   *
   * @default true
   */
  readonly playsinline?: boolean;

  /**
   * Native autoplay. Browsers usually require the root `defaultMuted` as well.
   *
   * @default false
   */
  readonly autoplay?: boolean;

  /**
   * Restart at the end.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Also render the browser's native controls.
   *
   * @default false
   */
  readonly controls?: boolean;

  /**
   * Native CORS policy, required for cross-origin captions.
   *
   * @default undefined
   */
  readonly crossOrigin?: MediaPlayerCrossOrigin;

  /**
   * Hide the browser picture-in-picture affordance and disable the PiP button.
   *
   * @default false
   */
  readonly disablePictureInPicture?: boolean;

  /**
   * Intrinsic width used to reserve layout space.
   *
   * @default undefined
   */
  readonly width?: number | string;

  /**
   * Intrinsic height used to reserve layout space.
   *
   * @default undefined
   */
  readonly height?: number | string;

  /**
   * Permit unencrypted `http:` sources for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;
}>();

defineSlots<{
  /** `<source>` and `<track>` children, plus fallback content. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const context = mediaPlayerContext.use();
const element = useTemplateRef<HTMLVideoElement>("element");

function safeSource(value: string | undefined, kind: "image" | "video"): string | undefined {
  if (value === undefined) return undefined;
  try {
    return normalizeMediaSource(value, { kind, allowInsecure });
  } catch {
    return undefined;
  }
}

const safeSrc = computed(() => safeSource(src, "video"));
const safePoster = computed(() => safeSource(poster, "image"));
const invalid = computed(() => src !== undefined && safeSrc.value === undefined);
// Validated sources are bound as one object; unsafe values never reach the DOM.
const sourceAttrs = computed(() => ({
  src: safeSrc.value,
  poster: safePoster.value,
}));
let unregister: (() => void) | null = null;

onMounted(() => {
  if (element.value !== null) unregister = context.registerMedia(element.value, "video");
});

onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

const exposed = { element } satisfies { readonly element: typeof element } & Omit<
  MediaPlayerMediaExpose<HTMLVideoElement>,
  "element"
>;

defineExpose(exposed);
</script>

<template>
  <!-- Caption <track> elements are supplied through the default slot. -->
  <!-- eslint-disable a11y/media-has-caption -->
  <video
    :id="context.getPartId('media')"
    ref="element"
    v-bind="sourceAttrs"
    :preload
    :playsinline
    :autoplay
    :loop
    :controls
    :muted="context.muted.value"
    :crossorigin="crossOrigin"
    :disablepictureinpicture="disablePictureInPicture ? '' : undefined"
    :width
    :height
    data-vize-ui="video-player-video"
    part="video"
    :data-state="context.state.value"
    :data-invalid-src="invalid ? 'true' : undefined"
  >
    <slot v-bind="context.slotState.value" />
  </video>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
