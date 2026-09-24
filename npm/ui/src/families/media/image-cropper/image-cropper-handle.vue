<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { imageCropperContext } from "./image-cropper-context.ts";
import { trackPointerDrag } from "./image-cropper-pointer.ts";
import type {
  CropArea,
  ImageCropperHandlePosition,
  ImageCropperPartExpose,
  ImageCropperSlotState,
} from "./image-cropper-types.ts";

const { position } = defineProps<{
  /** Edge or corner this handle resizes, by compass direction. @default required */
  readonly position: ImageCropperHandlePosition;
}>();

defineSlots<{
  /** Optional handle content. Receives the cropper state. */
  default(props: ImageCropperSlotState): unknown;
}>();

const context = imageCropperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
let stopDrag: (() => void) | null = null;

function onPointerDown(event: PointerEvent): void {
  const start: CropArea | null = context.crop.value;
  if (start === null || context.disabled.value || element.value === null) return;
  if (event.pointerType === "mouse" && event.button !== 0) return;
  event.preventDefault();
  event.stopPropagation();
  const scale = context.scale.value;
  context.setInteraction("resizing");
  stopDrag?.();
  stopDrag = trackPointerDrag(element.value, event, {
    onMove(dx, dy) {
      if (scale > 0) context.resizeFrom(start, position, dx / scale, dy / scale, "resize");
    },
    onEnd() {
      stopDrag = null;
      context.setInteraction("idle");
      context.commit();
    },
  });
}

// Keyboard resizing lives on the crop area (Alt/Ctrl + arrows); handles are pointer-only.
onMounted(() => element.value?.addEventListener("pointerdown", onPointerDown));

onBeforeUnmount(() => {
  stopDrag?.();
  element.value?.removeEventListener("pointerdown", onPointerDown);
});

type ImageCropperHandleSetupExpose = Omit<ImageCropperPartExpose<HTMLDivElement>, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ImageCropperHandleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="image-cropper-handle"
    part="handle"
    :data-position="position"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Position handles with [data-position] selectors. */
</style>
