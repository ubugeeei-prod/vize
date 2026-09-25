<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { normalizeMediaSource } from "../../../media/media-source.ts";
import { scrubberPreviewContext } from "./scrubber-preview-context.ts";
import type { ScrubberPreviewContextValue } from "./scrubber-preview-context.ts";
import { parseThumbnailVtt } from "./scrubber-preview-thumbnails.ts";
import type {
  ScrubberPreviewCue,
  ScrubberPreviewDirection,
  ScrubberPreviewKind,
  ScrubberPreviewRootExpose,
  ScrubberPreviewSlotState,
  ScrubberPreviewSprite,
} from "./scrubber-preview-types.ts";

const {
  duration,
  time = undefined,
  dir = "ltr",
  disabled = false,
  sprite = undefined,
  thumbnails = undefined,
  thumbnailsBaseUrl = undefined,
  videoSrc = undefined,
  captureWidth = 160,
  captureInterval = 1,
  cacheSize = 40,
} = defineProps<{
  /** Media duration in seconds. @default required */
  readonly duration: number;

  /**
   * Controlled preview time (`v-model:time`). `null` hides the preview; `undefined` is uncontrolled.
   *
   * @default undefined
   */
  readonly time?: number | null;

  /**
   * Reading direction; RTL tracks map the right edge to time zero.
   *
   * @default "ltr"
   */
  readonly dir?: ScrubberPreviewDirection;

  /**
   * Stop previewing and seeking.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Sprite-sheet thumbnails. Takes precedence over `thumbnails` and `videoSrc`.
   *
   * @default undefined
   */
  readonly sprite?: ScrubberPreviewSprite;

  /**
   * WebVTT thumbnails track text, or pre-parsed cues. Takes precedence over `videoSrc`.
   *
   * @default undefined
   */
  readonly thumbnails?: string | readonly ScrubberPreviewCue[];

  /**
   * Base URL that relative image URLs in VTT text resolve against.
   *
   * @default undefined
   */
  readonly thumbnailsBaseUrl?: string;

  /**
   * Video URL used to capture frames on the fly in the browser (same-origin or CORS-enabled).
   *
   * @default undefined
   */
  readonly videoSrc?: string;

  /**
   * Width of captured frames in pixels.
   *
   * @default 160
   */
  readonly captureWidth?: number;

  /**
   * Captured frames are shared across this many seconds (cache granularity).
   *
   * @default 1
   */
  readonly captureInterval?: number;

  /**
   * Maximum captured frames kept; older object URLs are revoked.
   *
   * @default 40
   */
  readonly cacheSize?: number;
}>();

const emit = defineEmits<{
  /** Fired when the preview time requests a new controlled value. */
  "update:time": [time: number | null];

  /** Fired when the user releases a click or scrub on the track. */
  seek: [time: number, nativeEvent: Event];
}>();

defineSlots<{
  /** Track, thumbnail, and time parts. Receives the preview state. */
  default(props: ScrubberPreviewSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const durationState = computed(() => (Number.isFinite(duration) && duration > 0 ? duration : 0));
const timeState = useControllableState<number | null>({
  value: () => time,
  defaultValue: null,
});
const currentTime = computed<number | null>(() => {
  const value = timeState.value.value;
  if (value === null || disabled) return null;
  return Math.min(durationState.value, Math.max(0, value));
});
const ratio = computed(() =>
  currentTime.value === null || durationState.value === 0
    ? null
    : currentTime.value / durationState.value,
);
const active = computed(() => currentTime.value !== null);
const cues = computed<readonly ScrubberPreviewCue[]>(() => {
  if (thumbnails === undefined) return [];
  return typeof thumbnails === "string"
    ? parseThumbnailVtt(thumbnails, thumbnailsBaseUrl)
    : thumbnails;
});
const safeVideoSrc = computed(() => {
  if (videoSrc === undefined) return undefined;
  try {
    return normalizeMediaSource(videoSrc, { kind: "video" });
  } catch {
    return undefined;
  }
});
const kind = computed<ScrubberPreviewKind>(() => {
  if (sprite !== undefined) return "sprite";
  if (thumbnails !== undefined) return "vtt";
  return safeVideoSrc.value === undefined ? "none" : "capture";
});
const slotState = computed<ScrubberPreviewSlotState>(() => ({
  active: active.value,
  duration: durationState.value,
  ratio: ratio.value,
  time: currentTime.value,
}));
const rootStyle = computed(() => ({
  "--vize-ui-scrubber-preview-ratio": String(ratio.value ?? 0),
}));

function setTime(next: number | null): boolean {
  const value = next === null || !Number.isFinite(next) ? null : next;
  if (value !== null && disabled) return false;
  if (Object.is(timeState.value.value, value)) return false;
  timeState.set(value);
  emit("update:time", value);
  return true;
}

scrubberPreviewContext.provide({
  active,
  cacheSize: computed(() => cacheSize),
  captureInterval: computed(() => captureInterval),
  captureWidth: computed(() => captureWidth),
  cues,
  dir: computed(() => dir),
  disabled: computed(() => disabled),
  duration: durationState,
  kind,
  previewRatio(next) {
    setTime(next === null ? null : next * durationState.value);
  },
  ratio,
  seekRatio(next, nativeEvent) {
    if (disabled) return;
    emit("seek", Math.min(1, Math.max(0, next)) * durationState.value, nativeEvent);
  },
  slotState,
  sprite: computed(() => sprite),
  time: currentTime,
  videoSrc: safeVideoSrc,
} satisfies ScrubberPreviewContextValue);

type ScrubberPreviewRootSetupExpose = Omit<
  ScrubberPreviewRootExpose,
  keyof ScrubberPreviewSlotState | "element"
> & {
  readonly active: ComputedRef<boolean>;
  readonly duration: ComputedRef<number>;
  readonly element: typeof element;
  readonly ratio: ComputedRef<number | null>;
  readonly time: ComputedRef<number | null>;
};

const exposed = {
  active,
  duration: durationState,
  element,
  ratio,
  setTime,
  time: currentTime,
} satisfies ScrubberPreviewRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="scrubber-preview-root"
    part="root"
    :data-state="active ? 'active' : 'idle'"
    :data-kind="kind"
    :data-disabled="disabled ? 'true' : undefined"
    :style="rootStyle"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Position the preview with --vize-ui-scrubber-preview-ratio (0..1). */
</style>
