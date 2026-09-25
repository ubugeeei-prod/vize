<script setup lang="ts">
import { onBeforeUnmount, onMounted, shallowRef, useTemplateRef } from "vue";
import type { ShallowRef } from "vue";

import { scrubberPreviewContext } from "./scrubber-preview-context.ts";
import { ratioFromPointer } from "./scrubber-preview-thumbnails.ts";
import type {
  ScrubberPreviewSlotState,
  ScrubberPreviewTrackExpose,
} from "./scrubber-preview-types.ts";

defineSlots<{
  /** Track contents, typically the consumer's seek bar. Receives the preview state. */
  default(props: ScrubberPreviewSlotState): unknown;
}>();

const context = scrubberPreviewContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const scrubbing = shallowRef(false);
let activePointer: number | null = null;

function ratioOf(event: PointerEvent): number | null {
  if (element.value === null) return null;
  return ratioFromPointer(event.clientX, element.value.getBoundingClientRect(), context.dir.value);
}

function onPointerMove(event: PointerEvent): void {
  if (context.disabled.value) return;
  if (activePointer !== null && event.pointerId !== activePointer) return;
  context.previewRatio(ratioOf(event));
}

function onPointerLeave(): void {
  if (activePointer === null) context.previewRatio(null);
}

function onPointerDown(event: PointerEvent): void {
  if (context.disabled.value || (event.pointerType === "mouse" && event.button !== 0)) return;
  activePointer = event.pointerId;
  scrubbing.value = true;
  try {
    element.value?.setPointerCapture(event.pointerId);
  } catch {
    // The pointer may already be gone; scrubbing still ends on pointerup.
  }
  context.previewRatio(ratioOf(event));
}

function finish(event: PointerEvent, seek: boolean): void {
  if (event.pointerId !== activePointer) return;
  activePointer = null;
  scrubbing.value = false;
  const ratio = ratioOf(event);
  if (seek && ratio !== null) context.seekRatio(ratio, event);
  // Touch and pen have no hover: hide the preview when the contact ends.
  if (event.pointerType !== "mouse") context.previewRatio(null);
}

function onPointerUp(event: PointerEvent): void {
  finish(event, true);
}

function onPointerCancel(event: PointerEvent): void {
  finish(event, false);
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  element.value?.addEventListener("pointermove", onPointerMove);
  element.value?.addEventListener("pointerenter", onPointerMove);
  element.value?.addEventListener("pointerleave", onPointerLeave);
  element.value?.addEventListener("pointerdown", onPointerDown);
  element.value?.addEventListener("pointerup", onPointerUp);
  element.value?.addEventListener("pointercancel", onPointerCancel);
});

onBeforeUnmount(() => {
  element.value?.removeEventListener("pointermove", onPointerMove);
  element.value?.removeEventListener("pointerenter", onPointerMove);
  element.value?.removeEventListener("pointerleave", onPointerLeave);
  element.value?.removeEventListener("pointerdown", onPointerDown);
  element.value?.removeEventListener("pointerup", onPointerUp);
  element.value?.removeEventListener("pointercancel", onPointerCancel);
});

type ScrubberPreviewTrackSetupExpose = Omit<ScrubberPreviewTrackExpose, "element" | "scrubbing"> & {
  readonly element: typeof element;
  readonly scrubbing: Readonly<ShallowRef<boolean>>;
};

const exposed = { element, scrubbing } satisfies ScrubberPreviewTrackSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="scrubber-preview-track"
    part="track"
    :data-state="context.active.value ? 'active' : 'idle'"
    :data-scrubbing="scrubbing ? 'true' : undefined"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Overlay the track on the seek bar; touch-action is consumer-owned. */
</style>
