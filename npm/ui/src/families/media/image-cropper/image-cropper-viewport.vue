<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { useSizeObserver } from "../../interaction/measure/measure-runtime.ts";
import { imageCropperContext } from "./image-cropper-context.ts";
import { fromViewport } from "./image-cropper-geometry.ts";
import { trackPointerDrag } from "./image-cropper-pointer.ts";
import type { ImageCropperPartExpose, ImageCropperSlotState } from "./image-cropper-types.ts";

const { wheelZoom = true } = defineProps<{
  /**
   * Zoom around the pointer with the mouse wheel or trackpad.
   *
   * @default true
   */
  readonly wheelZoom?: boolean;
}>();

defineSlots<{
  /** Image, crop area, and overlays. Receives the cropper state. */
  default(props: ImageCropperSlotState): unknown;
}>();

const context = imageCropperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
let stopDrag: (() => void) | null = null;

// Positioning mechanics are measurement-driven output; the custom properties
// let consumer CSS follow zoom, rotation, and scale.
const viewportStyle = computed(
  () =>
    ({
      "--vize-ui-image-cropper-rotation": `${context.rotation.value}deg`,
      "--vize-ui-image-cropper-scale": String(context.scale.value),
      "--vize-ui-image-cropper-zoom": String(context.zoom.value),
      overflow: "hidden",
      position: "relative",
      touchAction: "none",
    }) as const,
);

const sizeObserver = useSizeObserver({
  onResize(entries) {
    const latest = entries.at(-1);
    if (latest !== undefined) {
      context.setViewportSize({ width: latest.width, height: latest.height });
    }
  },
});

function localPoint(event: MouseEvent) {
  const rect = element.value?.getBoundingClientRect();
  return { x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) };
}

function onPointerDown(event: PointerEvent): void {
  if (!context.ready.value || context.disabled.value || element.value === null) return;
  if (event.pointerType === "mouse" && event.button !== 0) return;
  if (
    event.target instanceof Element &&
    event.target.closest('[data-vize-ui="image-cropper-area"]')
  ) {
    return;
  }
  event.preventDefault();
  const origin = context.center.value;
  const scale = context.scale.value;
  context.setInteraction("panning");
  stopDrag?.();
  stopDrag = trackPointerDrag(element.value, event, {
    onMove(dx, dy) {
      context.panTo({ x: origin.x - dx / scale, y: origin.y - dy / scale });
    },
    onEnd() {
      stopDrag = null;
      context.setInteraction("idle");
    },
  });
}

function onWheel(event: WheelEvent): void {
  if (!wheelZoom || !context.ready.value || context.disabled.value || event.deltaY === 0) return;
  event.preventDefault();
  const anchor = fromViewport(
    localPoint(event),
    context.center.value,
    context.viewportSize.value,
    context.scale.value,
  );
  context.zoomBy(event.deltaY < 0 ? 1 : -1, anchor);
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  if (element.value === null) return;
  const rect = element.value.getBoundingClientRect();
  context.setViewportSize({ width: rect.width, height: rect.height });
  sizeObserver.observe(element.value);
  element.value.addEventListener("pointerdown", onPointerDown);
  element.value.addEventListener("wheel", onWheel, { passive: false });
});

onBeforeUnmount(() => {
  stopDrag?.();
  element.value?.removeEventListener("pointerdown", onPointerDown);
  element.value?.removeEventListener("wheel", onWheel);
});

type ImageCropperViewportSetupExpose = Omit<ImageCropperPartExpose<HTMLDivElement>, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ImageCropperViewportSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="image-cropper-viewport"
    part="viewport"
    :data-ready="context.ready.value ? 'true' : undefined"
    :style="viewportStyle"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
