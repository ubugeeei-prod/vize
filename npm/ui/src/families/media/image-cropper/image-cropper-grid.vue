<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { imageCropperContext } from "./image-cropper-context.ts";
import type { ImageCropperPartExpose, ImageCropperSlotState } from "./image-cropper-types.ts";

const { divisions = 3 } = defineProps<{
  /**
   * Number of equal columns and rows; `3` draws the rule of thirds.
   *
   * @default 3
   */
  readonly divisions?: number;
}>();

defineSlots<{
  /** Optional extra overlay content. Receives the cropper state. */
  default(props: ImageCropperSlotState): unknown;
}>();

interface GridLine {
  readonly key: string;
  readonly axis: "x" | "y";
  readonly style: Readonly<Record<string, string>>;
}

const context = imageCropperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const lines = computed<readonly GridLine[]>(() => {
  const count = Math.max(1, Math.floor(Number.isFinite(divisions) ? divisions : 3));
  const result: GridLine[] = [];
  for (const axis of ["x", "y"] as const) {
    for (let index = 1; index < count; index += 1) {
      const offset = `${Math.round((index / count) * 100000) / 1000}%`;
      result.push({
        axis,
        key: `${axis}-${index}`,
        style: { "--vize-ui-image-cropper-grid-offset": offset },
      });
    }
  }
  return result;
});

type ImageCropperGridSetupExpose = Omit<ImageCropperPartExpose<HTMLDivElement>, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies ImageCropperGridSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="image-cropper-grid"
    part="grid"
    :data-interaction="context.slotState.value.interaction"
  >
    <span
      v-for="line in lines as readonly GridLine[]"
      :key="line.key"
      data-vize-ui="image-cropper-grid-line"
      :data-axis="line.axis"
      :style="line.style"
    ></span>
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Place lines with var(--vize-ui-image-cropper-grid-offset). */
</style>
