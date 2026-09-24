<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import {
  bufferedEnd,
  captureMediaPointer,
  clampMediaValue,
  formatMediaTime,
  pointerRatio,
  releaseMediaPointer,
  resolveSliderKey,
  toPercent,
} from "./media-player-format.ts";
import type { MediaPlayerSliderExpose, MediaPlayerSlotState } from "./media-player-types.ts";

const { ariaLabel = undefined, ariaLabelledby = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to the root `messages.seek`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that name the slider instead of `aria-label`.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Thumb content. Receives the playback state. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const context = mediaPlayerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const thumb = useTemplateRef<HTMLDivElement>("thumb");
const activePointer = shallowRef<number | null>(null);
const dragging = computed(() => activePointer.value !== null);
const duration = computed(() => context.slotState.value.duration);
const current = computed(() =>
  clampMediaValue(context.slotState.value.currentTime, 0, duration.value),
);
const disabled = computed(
  () => context.media.value === null || !(duration.value > 0) || !Number.isFinite(duration.value),
);
const label = computed(() =>
  ariaLabelledby === undefined ? (ariaLabel ?? context.messages.value.seek) : ariaLabel,
);
const valueText = computed(() =>
  context.messages.value.seekValueText(
    formatMediaTime(current.value, duration.value),
    formatMediaTime(duration.value),
  ),
);
const trackStyle = computed(() => ({
  "--vize-ui-media-player-progress": `${toPercent(current.value, duration.value)}%`,
  "--vize-ui-media-player-buffered": `${toPercent(
    bufferedEnd(context.slotState.value.buffered, current.value),
    duration.value,
  )}%`,
}));

function seekToPointer(event: PointerEvent): void {
  if (element.value === null) return;
  const rect = element.value.getBoundingClientRect();
  const ratio = pointerRatio(rect, event.clientX, context.dir.value === "rtl");
  context.seek(ratio * duration.value, "pointer");
}

function onPointerdown(event: PointerEvent): void {
  if (disabled.value || event.button !== 0 || element.value === null) return;
  event.preventDefault();
  activePointer.value = event.pointerId;
  captureMediaPointer(element.value, event.pointerId);
  thumb.value?.focus({ preventScroll: true });
  context.setScrubbing(true);
  seekToPointer(event);
}

function onPointermove(event: PointerEvent): void {
  if (activePointer.value === event.pointerId) seekToPointer(event);
}

function onPointerup(event: PointerEvent): void {
  if (activePointer.value !== event.pointerId) return;
  activePointer.value = null;
  if (element.value !== null) releaseMediaPointer(element.value, event.pointerId);
  context.setScrubbing(false);
}

function onKeydown(event: KeyboardEvent): void {
  const intent = resolveSliderKey(event.key, context.dir.value === "rtl");
  if (intent === null || disabled.value) return;
  event.preventDefault();
  if (intent.kind === "edge") {
    context.seek(intent.edge === "min" ? 0 : duration.value, "keyboard");
    return;
  }
  const amount = intent.kind === "page" ? duration.value / 10 : context.seekStep.value;
  context.seek(current.value + amount * intent.sign, "keyboard");
}

// Role, tabindex, and keyboard handling are bound together; disabled sliders leave the tab order.
const thumbProps = computed<{
  readonly role: "slider";
  readonly tabindex: -1 | 0;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ role: "slider", tabindex: disabled.value ? -1 : 0, onKeydown }));

function focus(options?: FocusOptions): void {
  thumb.value?.focus(options);
}

type SetupExpose = Omit<MediaPlayerSliderExpose, "dragging" | "element" | "thumb"> & {
  readonly dragging: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly thumb: typeof thumb;
};

const exposed = { dragging, element, focus, thumb } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="media-player-seek-slider"
    part="seek-slider"
    :data-state="context.state.value"
    :data-dragging="dragging ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :style="trackStyle"
    @pointerdown="onPointerdown"
    @pointermove="onPointermove"
    @pointerup="onPointerup"
    @pointercancel="onPointerup"
    @lostpointercapture="onPointerup"
  >
    <div
      :id="context.getPartId('seek')"
      ref="thumb"
      v-bind="thumbProps"
      :aria-label="label"
      :aria-labelledby="ariaLabelledby"
      aria-orientation="horizontal"
      :aria-valuemin="0"
      :aria-valuemax="Math.floor(duration)"
      :aria-valuenow="Math.floor(current)"
      :aria-valuetext="valueText"
      :aria-disabled="disabled ? 'true' : undefined"
      data-vize-ui="media-player-seek-thumb"
      part="seek-thumb"
    >
      <slot v-bind="context.slotState.value" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
