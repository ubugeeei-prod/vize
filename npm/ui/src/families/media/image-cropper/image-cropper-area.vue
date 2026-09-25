<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { imageCropperContext } from "./image-cropper-context.ts";
import { toViewport } from "./image-cropper-geometry.ts";
import { trackPointerDrag } from "./image-cropper-pointer.ts";
import type {
  CropArea,
  ImageCropperHandlePosition,
  ImageCropperPartExpose,
  ImageCropperSlotState,
} from "./image-cropper-types.ts";

defineSlots<{
  /** Handles, grid, and other overlays inside the crop box. Receives the cropper state. */
  default(props: ImageCropperSlotState): unknown;
}>();

const context = imageCropperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
let stopDrag: (() => void) | null = null;

function px(value: number): string {
  return `${Math.round(value * 100) / 100}px`;
}

const label = computed(() =>
  context.crop.value === null ? undefined : context.messages.value.area(context.crop.value),
);

// Geometry is applied only once measured, so server and hydration markup agree.
const areaStyle = computed(() => {
  const crop = context.crop.value;
  if (!context.ready.value || crop === null) return undefined;
  const scale = context.scale.value;
  const origin = toViewport(crop, context.center.value, context.viewportSize.value, scale);
  return {
    "--vize-ui-image-cropper-crop-height": String(crop.height),
    "--vize-ui-image-cropper-crop-width": String(crop.width),
    "--vize-ui-image-cropper-crop-x": String(crop.x),
    "--vize-ui-image-cropper-crop-y": String(crop.y),
    boxSizing: "border-box",
    height: px(crop.height * scale),
    left: px(origin.x),
    position: "absolute",
    top: px(origin.y),
    width: px(crop.width * scale),
  } as const;
});

function onPointerDown(event: PointerEvent): void {
  const start: CropArea | null = context.crop.value;
  if (start === null || context.disabled.value || element.value === null) return;
  if (event.pointerType === "mouse" && event.button !== 0) return;
  if (
    event.target instanceof Element &&
    event.target.closest('[data-vize-ui="image-cropper-handle"]')
  ) {
    return;
  }
  event.preventDefault();
  element.value.focus({ preventScroll: true });
  const scale = context.scale.value;
  context.setInteraction("moving");
  stopDrag?.();
  stopDrag = trackPointerDrag(element.value, event, {
    onMove(dx, dy) {
      if (scale > 0) context.moveFrom(start, dx / scale, dy / scale, "move");
    },
    onEnd() {
      stopDrag = null;
      context.setInteraction("idle");
      context.commit();
    },
  });
}

const resizeKeys: Readonly<Record<string, readonly [ImageCropperHandlePosition, number, number]>> =
  {
    ArrowDown: ["s", 0, 1],
    ArrowLeft: ["e", -1, 0],
    ArrowRight: ["e", 1, 0],
    ArrowUp: ["s", 0, -1],
  };

const moveKeys: Readonly<Record<string, readonly [number, number]>> = {
  ArrowDown: [0, 1],
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
};

function onKeydown(event: KeyboardEvent): void {
  const crop = context.crop.value;
  if (crop === null || context.disabled.value || event.target !== element.value) return;
  const step = context.nudgeStep.value * (event.shiftKey ? 10 : 1);
  const resize = resizeKeys[event.key];
  const move = moveKeys[event.key];
  const anchor = { x: crop.x + crop.width / 2, y: crop.y + crop.height / 2 };
  let handled = true;
  if ((event.altKey || event.ctrlKey || event.metaKey) && resize !== undefined) {
    context.resizeFrom(crop, resize[0], resize[1] * step, resize[2] * step, "keyboard");
  } else if (move !== undefined) {
    context.moveFrom(crop, move[0] * step, move[1] * step, "keyboard");
  } else if (event.key === "+" || event.key === "=") {
    context.zoomBy(1, anchor);
  } else if (event.key === "-" || event.key === "_") {
    context.zoomBy(-1, anchor);
  } else if (event.key === "]") {
    context.rotateBy(1);
  } else if (event.key === "[") {
    context.rotateBy(-1);
  } else {
    handled = false;
  }
  if (!handled) return;
  event.preventDefault();
  context.commit();
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  element.value?.addEventListener("pointerdown", onPointerDown);
  element.value?.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  stopDrag?.();
  element.value?.removeEventListener("pointerdown", onPointerDown);
  element.value?.removeEventListener("keydown", onKeydown);
});

type ImageCropperAreaSetupExpose = Omit<ImageCropperPartExpose<HTMLDivElement>, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ImageCropperAreaSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="group"
    :tabindex="context.disabled.value ? -1 : 0"
    :aria-roledescription="context.messages.value.areaRoleDescription"
    :aria-label="label"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    data-vize-ui="image-cropper-area"
    part="area"
    :data-interaction="context.slotState.value.interaction"
    :style="areaStyle"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
