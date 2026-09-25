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
  preload = "metadata",
  autoplay = false,
  loop = false,
  controls = false,
  crossOrigin = undefined,
  allowInsecure = false,
} = defineProps<{
  /**
   * Audio source, validated by the shared media-source policy. Unsafe sources are
   * dropped (`data-invalid-src`); use `<source>` children for format fallbacks.
   *
   * @default undefined
   */
  readonly src?: string;

  /**
   * Native preload hint.
   *
   * @default "metadata"
   */
  readonly preload?: MediaPlayerPreload;

  /**
   * Native autoplay. Browsers may block audible autoplay until the user interacts.
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
const element = useTemplateRef<HTMLAudioElement>("element");

function safeSource(value: string | undefined, kind: "audio"): string | undefined {
  if (value === undefined) return undefined;
  try {
    return normalizeMediaSource(value, { kind, allowInsecure });
  } catch {
    return undefined;
  }
}

const safeSrc = computed(() => safeSource(src, "audio"));
const invalid = computed(() => src !== undefined && safeSrc.value === undefined);
// Validated sources are bound as one object; unsafe values never reach the DOM.
const sourceAttrs = computed(() => ({
  src: safeSrc.value,
}));
let unregister: (() => void) | null = null;

onMounted(() => {
  if (element.value !== null) unregister = context.registerMedia(element.value, "audio");
});

onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

const exposed = { element } satisfies { readonly element: typeof element } & Omit<
  MediaPlayerMediaExpose<HTMLAudioElement>,
  "element"
>;

defineExpose(exposed);
</script>

<template>
  <!-- Caption <track> elements are supplied through the default slot. -->
  <!-- eslint-disable a11y/media-has-caption -->
  <audio
    :id="context.getPartId('media')"
    ref="element"
    v-bind="sourceAttrs"
    :preload
    :autoplay
    :loop
    :controls
    :muted="context.muted.value"
    :crossorigin="crossOrigin"
    data-vize-ui="audio-player-audio"
    part="audio"
    :data-state="context.state.value"
    :data-invalid-src="invalid ? 'true' : undefined"
  >
    <slot v-bind="context.slotState.value" />
  </audio>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
