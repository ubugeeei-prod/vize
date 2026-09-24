<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import {
  captureMediaPointer,
  pointerRatio,
  releaseMediaPointer,
  resolveSliderKey,
} from "./media-player-format.ts";
import type { MediaPlayerSliderExpose, MediaPlayerSlotState } from "./media-player-types.ts";

const { ariaLabel = undefined, ariaLabelledby = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to the root `messages.volume`.
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

const PAGE_STEP = 0.1;

const context = mediaPlayerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const thumb = useTemplateRef<HTMLDivElement>("thumb");
const activePointer = shallowRef<number | null>(null);
const dragging = computed(() => activePointer.value !== null);
const volume = computed(() => context.slotState.value.volume);
const percent = computed(() => Math.round(volume.value * 100));
const audible = computed(() => (context.muted.value ? 0 : percent.value));
const label = computed(() =>
  ariaLabelledby === undefined ? (ariaLabel ?? context.messages.value.volume) : ariaLabel,
);
const trackStyle = computed(() => ({
  "--vize-ui-media-player-volume": `${audible.value}%`,
}));

function setFromPointer(event: PointerEvent): void {
  if (element.value === null) return;
  const rect = element.value.getBoundingClientRect();
  context.setVolume(pointerRatio(rect, event.clientX, context.dir.value === "rtl"));
}

function onPointerdown(event: PointerEvent): void {
  if (event.button !== 0 || element.value === null) return;
  event.preventDefault();
  activePointer.value = event.pointerId;
  captureMediaPointer(element.value, event.pointerId);
  thumb.value?.focus({ preventScroll: true });
  setFromPointer(event);
}

function onPointermove(event: PointerEvent): void {
  if (activePointer.value === event.pointerId) setFromPointer(event);
}

function onPointerup(event: PointerEvent): void {
  if (activePointer.value !== event.pointerId) return;
  activePointer.value = null;
  if (element.value !== null) releaseMediaPointer(element.value, event.pointerId);
}

function onKeydown(event: KeyboardEvent): void {
  const intent = resolveSliderKey(event.key, context.dir.value === "rtl");
  if (intent === null) return;
  event.preventDefault();
  if (intent.kind === "edge") {
    context.setVolume(intent.edge === "min" ? 0 : 1);
    return;
  }
  const amount = intent.kind === "page" ? PAGE_STEP : context.volumeStep.value;
  context.setVolume(volume.value + amount * intent.sign);
}

// Role, tabindex, and keyboard handling are bound together.
const thumbProps = computed<{
  readonly role: "slider";
  readonly tabindex: 0;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ role: "slider", tabindex: 0, onKeydown }));

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
    data-vize-ui="media-player-volume-slider"
    part="volume-slider"
    :data-muted="context.muted.value ? 'true' : undefined"
    :data-dragging="dragging ? 'true' : undefined"
    :style="trackStyle"
    @pointerdown="onPointerdown"
    @pointermove="onPointermove"
    @pointerup="onPointerup"
    @pointercancel="onPointerup"
    @lostpointercapture="onPointerup"
  >
    <div
      :id="context.getPartId('volume')"
      ref="thumb"
      v-bind="thumbProps"
      :aria-label="label"
      :aria-labelledby="ariaLabelledby"
      aria-orientation="horizontal"
      :aria-valuemin="0"
      :aria-valuemax="100"
      :aria-valuenow="percent"
      :aria-valuetext="context.messages.value.volumeValueText(percent, context.muted.value)"
      data-vize-ui="media-player-volume-thumb"
      part="volume-thumb"
    >
      <slot v-bind="context.slotState.value" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
