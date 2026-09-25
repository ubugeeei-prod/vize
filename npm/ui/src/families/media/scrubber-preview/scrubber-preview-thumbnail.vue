<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { normalizeMediaSource } from "../../../media/media-source.ts";
import { captureVideoFrame, ScrubberPreviewError } from "./scrubber-preview-capture.ts";
import { scrubberPreviewContext } from "./scrubber-preview-context.ts";
import {
  createFrameCache,
  findThumbnailCue,
  quantizeTime,
  spriteFrame,
} from "./scrubber-preview-thumbnails.ts";
import type { FrameCache } from "./scrubber-preview-thumbnails.ts";
import type {
  ScrubberPreviewErrorCode,
  ScrubberPreviewFrame,
  ScrubberPreviewKind,
  ScrubberPreviewStatus,
  ScrubberPreviewThumbnailExpose,
  ScrubberPreviewThumbnailSlotState,
} from "./scrubber-preview-types.ts";

defineSlots<{
  /**
   * Custom thumbnail rendering. Defaults to an `<img>` for captured frames;
   * sprite and VTT frames are published as custom properties for CSS backgrounds.
   */
  default(props: ScrubberPreviewThumbnailSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when a thumbnail cannot be produced. */
  error: [error: ScrubberPreviewError];
}>();

const context = scrubberPreviewContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const captured = shallowRef<ScrubberPreviewFrame | null>(null);
const capturing = shallowRef(false);
const captureError = shallowRef<ScrubberPreviewErrorCode | null>(null);
let video: HTMLVideoElement | null = null;
let cache: FrameCache | null = null;
let busy = false;
let pendingKey: number | null = null;
let requestedKey: number | null = null;

function safeImage(src: string): string | null {
  try {
    return normalizeMediaSource(src, { kind: "image" });
  } catch {
    return null;
  }
}

const staticFrame = computed<ScrubberPreviewFrame | null | "invalid">(() => {
  const time = context.time.value;
  if (time === null) return null;
  if (context.kind.value === "sprite" && context.sprite.value !== undefined) {
    const frame = spriteFrame(context.sprite.value, time);
    const src = safeImage(frame.src);
    return src === null ? "invalid" : { ...frame, src };
  }
  if (context.kind.value === "vtt") {
    const cue = findThumbnailCue(context.cues.value, time);
    if (cue === undefined) return null;
    const src = safeImage(cue.src);
    return src === null ? "invalid" : { src, region: cue.region };
  }
  return null;
});

const frame = computed<ScrubberPreviewFrame | null>(() => {
  if (context.kind.value === "capture") return context.time.value === null ? null : captured.value;
  return staticFrame.value === "invalid" ? null : staticFrame.value;
});

const error = computed<ScrubberPreviewErrorCode | null>(() => {
  if (context.kind.value === "capture") return captureError.value;
  return staticFrame.value === "invalid" ? "VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED" : null;
});

const status = computed<ScrubberPreviewStatus>(() => {
  if (context.time.value === null) return "idle";
  if (error.value !== null) return "error";
  if (frame.value !== null) return "ready";
  return context.kind.value === "capture" && capturing.value ? "loading" : "idle";
});

const kind = computed<ScrubberPreviewKind>(() => context.kind.value);
const slotState = computed<ScrubberPreviewThumbnailSlotState>(() => ({
  error: error.value,
  frame: frame.value,
  kind: kind.value,
  status: status.value,
  time: context.time.value,
}));
function styleOf(current: ScrubberPreviewFrame | null): Record<string, string> | undefined {
  if (current === null) return undefined;
  const style: Record<string, string> = {
    "--vize-ui-scrubber-preview-image": `url(${JSON.stringify(current.src)})`,
  };
  if (current.region !== null) {
    style["--vize-ui-scrubber-preview-x"] = `${-current.region.x}px`;
    style["--vize-ui-scrubber-preview-y"] = `${-current.region.y}px`;
    style["--vize-ui-scrubber-preview-width"] = `${current.region.width}px`;
    style["--vize-ui-scrubber-preview-height"] = `${current.region.height}px`;
  }
  return style;
}

const frameStyle = computed(() => styleOf(frame.value));
// Captured frames are same-document object URLs; sprite and VTT images passed the media-source policy.
const imageProps = computed(() =>
  kind.value === "capture" && frame.value !== null ? { src: frame.value.src } : undefined,
);

function report(code: ScrubberPreviewErrorCode, cause: unknown): void {
  captureError.value = code;
  emit(
    "error",
    cause instanceof ScrubberPreviewError ? cause : new ScrubberPreviewError(code, { cause }),
  );
}

async function capture(key: number): Promise<void> {
  if (video === null || cache === null) return;
  busy = true;
  capturing.value = true;
  try {
    const blob = await captureVideoFrame(video, key, { width: context.captureWidth.value });
    if (cache === null) return;
    const url = URL.createObjectURL(blob);
    cache.set(key, url);
    captureError.value = null;
    if (requestedKey === key) captured.value = { src: url, region: null };
  } catch (cause) {
    report(
      cause instanceof ScrubberPreviewError
        ? cause.code
        : "VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED",
      cause,
    );
  } finally {
    busy = false;
    capturing.value = false;
  }
  // Latest wins: only the most recent request made while busy is captured next.
  const next = pendingKey;
  pendingKey = null;
  if (next !== null && next === requestedKey && cache?.get(next) === undefined) {
    await capture(next);
  }
}

function request(time: number | null): void {
  if (context.kind.value !== "capture" || time === null || cache === null) {
    requestedKey = null;
    return;
  }
  const key = quantizeTime(time, context.captureInterval.value);
  requestedKey = key;
  const cached = cache.get(key);
  if (cached !== undefined) {
    captured.value = { src: cached, region: null };
    return;
  }
  if (busy) {
    pendingKey = key;
    return;
  }
  void capture(key);
}

function onVideoError(event: Event): void {
  report("VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED", event);
}

function releaseVideo(): void {
  cache?.clear();
  cache = null;
  captured.value = null;
  if (video !== null) {
    video.removeEventListener("error", onVideoError);
    video.removeAttribute("src");
    video = null;
  }
}

function prepareVideo(src: string | undefined): void {
  releaseVideo();
  captureError.value = null;
  if (context.kind.value !== "capture" || src === undefined) return;
  cache = createFrameCache(context.cacheSize.value, (url) => URL.revokeObjectURL(url));
  video = document.createElement("video");
  video.muted = true;
  video.preload = "auto";
  video.crossOrigin = "anonymous";
  video.playsInline = true;
  video.addEventListener("error", onVideoError);
  video.src = src;
  request(context.time.value);
}

// The capture video exists only on the client, so server markup never references it.
let mounted = false;
onMounted(() => {
  mounted = true;
  prepareVideo(context.videoSrc.value);
});
watch([context.videoSrc, context.kind], ([src]) => {
  if (mounted) prepareVideo(src);
});
watch(context.time, (time) => {
  if (mounted) request(time);
});

onBeforeUnmount(() => {
  mounted = false;
  releaseVideo();
});

type ScrubberPreviewThumbnailSetupExpose = Omit<
  ScrubberPreviewThumbnailExpose,
  keyof ScrubberPreviewThumbnailSlotState | "element"
> & {
  readonly element: typeof element;
  readonly error: ComputedRef<ScrubberPreviewErrorCode | null>;
  readonly frame: ComputedRef<ScrubberPreviewFrame | null>;
  readonly kind: ComputedRef<ScrubberPreviewKind>;
  readonly status: ComputedRef<ScrubberPreviewStatus>;
  readonly time: ComputedRef<number | null>;
};

const exposed = {
  element,
  error,
  frame,
  kind,
  status,
  time: context.time,
} satisfies ScrubberPreviewThumbnailSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    :hidden="context.active.value ? undefined : true"
    data-vize-ui="scrubber-preview-thumbnail"
    part="thumbnail"
    :data-kind="kind"
    :data-status="status"
    :style="frameStyle"
  >
    <slot v-bind="slotState">
      <img
        v-if="imageProps"
        v-bind="imageProps"
        alt=""
        data-vize-ui="scrubber-preview-image"
        part="image"
      />
    </slot>
  </div>
</template>

<style scoped>
/* Headless by design. For sprites: background: var(--vize-ui-scrubber-preview-image)
   var(--vize-ui-scrubber-preview-x) var(--vize-ui-scrubber-preview-y); */
</style>
