<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { ImageCropperError, cropImage } from "./image-cropper-canvas.ts";
import { imageCropperContext } from "./image-cropper-context.ts";
import type { ImageCropperContextValue } from "./image-cropper-context.ts";
import {
  centeredCrop,
  clampCrop,
  clampViewCenter,
  fitScale,
  moveCrop,
  normalizeRotation,
  refitCrop,
  resizeCrop,
  rotatedBounds,
  zoomAroundPoint,
} from "./image-cropper-geometry.ts";
import type {
  CropArea,
  CropConstraints,
  CropImageOptions,
  CropPoint,
  CropSize,
  ImageCropperChangeReason,
  ImageCropperHandlePosition,
  ImageCropperInteraction,
  ImageCropperMessages,
  ImageCropperRootExpose,
  ImageCropperSlotState,
} from "./image-cropper-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  zoom = undefined,
  defaultZoom = 1,
  rotation = undefined,
  defaultRotation = 0,
  aspectRatio = undefined,
  minWidth = undefined,
  minHeight = undefined,
  maxWidth = undefined,
  maxHeight = undefined,
  minZoom = 1,
  maxZoom = 5,
  zoomStep = 0.1,
  rotationStep = 90,
  nudgeStep = 1,
  autoCropArea = 0.8,
  disabled = false,
  messages = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled crop in rotated-image pixels (`v-model`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: CropArea;

  /**
   * Initial crop for uncontrolled use. `undefined` centers a crop covering `autoCropArea`.
   *
   * @default undefined
   */
  readonly defaultValue?: CropArea;

  /**
   * Controlled zoom factor (`v-model:zoom`); `1` fits the whole image.
   *
   * @default undefined
   */
  readonly zoom?: number;

  /**
   * Initial zoom for uncontrolled use.
   *
   * @default 1
   */
  readonly defaultZoom?: number;

  /**
   * Controlled rotation in degrees (`v-model:rotation`).
   *
   * @default undefined
   */
  readonly rotation?: number;

  /**
   * Initial rotation for uncontrolled use.
   *
   * @default 0
   */
  readonly defaultRotation?: number;

  /**
   * Locked width / height ratio. `undefined` allows free resizing.
   *
   * @default undefined
   */
  readonly aspectRatio?: number;

  /**
   * Minimum crop width in image pixels.
   *
   * @default undefined
   */
  readonly minWidth?: number;

  /**
   * Minimum crop height in image pixels.
   *
   * @default undefined
   */
  readonly minHeight?: number;

  /**
   * Maximum crop width in image pixels.
   *
   * @default undefined
   */
  readonly maxWidth?: number;

  /**
   * Maximum crop height in image pixels.
   *
   * @default undefined
   */
  readonly maxHeight?: number;

  /**
   * Smallest zoom factor.
   *
   * @default 1
   */
  readonly minZoom?: number;

  /**
   * Largest zoom factor.
   *
   * @default 5
   */
  readonly maxZoom?: number;

  /**
   * Relative zoom change per key press or wheel notch.
   *
   * @default 0.1
   */
  readonly zoomStep?: number;

  /**
   * Degrees rotated by `[` and `]`.
   *
   * @default 90
   */
  readonly rotationStep?: number;

  /**
   * Image pixels moved per arrow key press (Shift multiplies by 10).
   *
   * @default 1
   */
  readonly nudgeStep?: number;

  /**
   * Fraction of the image the initial centered crop covers.
   *
   * @default 0.8
   */
  readonly autoCropArea?: number;

  /**
   * Suppress every pointer and keyboard interaction.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Localized accessible text. Missing keys fall back to English.
   *
   * @default undefined
   */
  readonly messages?: Partial<ImageCropperMessages>;
}>();

const emit = defineEmits<{
  /** Fired when the crop requests a new controlled value. */
  "update:modelValue": [crop: CropArea];

  /** Fired when zoom requests a new controlled value. */
  "update:zoom": [zoom: number];

  /** Fired when rotation requests a new controlled value. */
  "update:rotation": [rotation: number];

  /** Fired after every distinct crop request, with its cause. */
  change: [crop: CropArea, reason: ImageCropperChangeReason];

  /** Fired when a pointer interaction or key press finishes changing the crop. */
  cropEnd: [crop: CropArea];
}>();

defineSlots<{
  /** Viewport, image, area, handles, and controls. Receives the cropper state. */
  default(props: ImageCropperSlotState): unknown;
}>();

const defaultMessages: ImageCropperMessages = {
  area: (crop) =>
    `Crop area ${Math.round(crop.width)} by ${Math.round(crop.height)} pixels at ${Math.round(crop.x)}, ${Math.round(crop.y)}`,
  areaRoleDescription: "crop area",
};

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "image-cropper" });
const naturalSize = shallowRef<CropSize | null>(null);
const viewportSize = shallowRef<CropSize>({ width: 0, height: 0 });
const interaction = shallowRef<ImageCropperInteraction>("idle");
const viewCenter = shallowRef<CropPoint | null>(null);
const cropState = useControllableState<CropArea | null>({
  value: () => modelValue,
  defaultValue: () => defaultValue ?? null,
});
const zoomState = useControllableState<number>({
  value: () => zoom,
  defaultValue: () => defaultZoom,
});
const rotationState = useControllableState<number>({
  value: () => rotation,
  defaultValue: () => defaultRotation,
});
const constraints = computed<CropConstraints>(() => ({
  ...(aspectRatio === undefined ? {} : { aspectRatio }),
  ...(maxHeight === undefined ? {} : { maxHeight }),
  ...(maxWidth === undefined ? {} : { maxWidth }),
  ...(minHeight === undefined ? {} : { minHeight }),
  ...(minWidth === undefined ? {} : { minWidth }),
}));
const zoomRange = computed(() => {
  const low = Number.isFinite(minZoom) && minZoom > 0 ? minZoom : 1;
  return { low, high: Math.max(low, Number.isFinite(maxZoom) ? maxZoom : low) };
});
const zoomValue = computed(() => clampZoom(zoomState.value.value));
const rotationValue = computed(() => normalizeRotation(rotationState.value.value));
const bounds = computed(() =>
  naturalSize.value === null ? null : rotatedBounds(naturalSize.value, rotationValue.value),
);
const crop = computed<CropArea | null>(() => {
  if (bounds.value === null) return null;
  const raw = cropState.value.value ?? centeredCrop(bounds.value, constraints.value, autoCropArea);
  return clampCrop(raw, bounds.value, constraints.value);
});
const scale = computed(() =>
  bounds.value === null ? 0 : fitScale(viewportSize.value, bounds.value) * zoomValue.value,
);
const center = computed<CropPoint>(() => {
  if (bounds.value === null) return { x: 0, y: 0 };
  const requested = viewCenter.value ?? { x: bounds.value.width / 2, y: bounds.value.height / 2 };
  return clampViewCenter(requested, bounds.value, viewportSize.value, scale.value);
});
const ready = computed(() => bounds.value !== null && scale.value > 0);
const messageSet = computed<ImageCropperMessages>(() => ({
  area: messages?.area ?? defaultMessages.area,
  areaRoleDescription: messages?.areaRoleDescription ?? defaultMessages.areaRoleDescription,
}));
const slotState = computed<ImageCropperSlotState>(() => ({
  crop: crop.value,
  disabled,
  interaction: interaction.value,
  naturalSize: naturalSize.value,
  ready: ready.value,
  rotation: rotationValue.value,
  zoom: zoomValue.value,
}));

function clampZoom(value: number): number {
  const { low, high } = zoomRange.value;
  return Math.min(high, Math.max(low, Number.isFinite(value) ? value : low));
}

function sameCrop(left: CropArea | null, right: CropArea): boolean {
  return (
    left !== null &&
    left.x === right.x &&
    left.y === right.y &&
    left.width === right.width &&
    left.height === right.height
  );
}

function setCrop(next: CropArea, reason: ImageCropperChangeReason): boolean {
  if (bounds.value === null) return false;
  const clamped = Object.freeze(clampCrop(next, bounds.value, constraints.value));
  if (sameCrop(crop.value, clamped) && reason !== "init") return false;
  cropState.set(clamped);
  emit("update:modelValue", clamped);
  emit("change", clamped, reason);
  return true;
}

function moveFrom(start: CropArea, dx: number, dy: number, reason: ImageCropperChangeReason) {
  if (bounds.value === null || disabled) return false;
  return setCrop(moveCrop(start, dx, dy, bounds.value), reason);
}

function resizeFrom(
  start: CropArea,
  handle: ImageCropperHandlePosition,
  dx: number,
  dy: number,
  reason: ImageCropperChangeReason,
): boolean {
  if (bounds.value === null || disabled) return false;
  return setCrop(resizeCrop(start, handle, dx, dy, bounds.value, constraints.value), reason);
}

function currentZoom(): number {
  return zoomValue.value;
}

function currentRotation(): number {
  return rotationValue.value;
}

function zoomTo(next: number, anchor?: CropPoint): boolean {
  const target = clampZoom(next);
  if (target === currentZoom()) return false;
  if (anchor !== undefined && bounds.value !== null) {
    const base = fitScale(viewportSize.value, bounds.value);
    viewCenter.value = zoomAroundPoint(center.value, anchor, base * currentZoom(), base * target);
  }
  zoomState.set(target);
  emit("update:zoom", target);
  return true;
}

function zoomBy(steps: number, anchor?: CropPoint): boolean {
  if (disabled) return false;
  const factor = Math.pow(1 + Math.max(0, zoomStep), steps);
  return zoomTo(zoomValue.value * factor, anchor);
}

function setRotation(degrees: number): boolean {
  const target = normalizeRotation(degrees);
  if (target === currentRotation()) return false;
  const refit =
    crop.value !== null && naturalSize.value !== null
      ? Object.freeze(
          refitCrop(
            crop.value,
            rotatedBounds(naturalSize.value, currentRotation()),
            rotatedBounds(naturalSize.value, target),
            constraints.value,
          ),
        )
      : null;
  rotationState.set(target);
  emit("update:rotation", target);
  if (refit !== null) {
    cropState.set(refit);
    emit("update:modelValue", refit);
    emit("change", refit, "rotate");
  }
  viewCenter.value = null;
  return true;
}

function rotateBy(steps: number): boolean {
  if (disabled) return false;
  return setRotation(rotationValue.value + steps * rotationStep);
}

function commit(): void {
  if (crop.value !== null) emit("cropEnd", crop.value);
}

function reset(): void {
  zoomTo(1);
  setRotation(0);
  viewCenter.value = null;
  if (bounds.value !== null) {
    setCrop(defaultValue ?? centeredCrop(bounds.value, constraints.value, autoCropArea), "api");
  }
}

// Publish the initial crop once the image size is known so controlled parents
// and form state receive the same area the cropper displays.
watch(naturalSize, (next, previous) => {
  if (next === null || bounds.value === null) return;
  const changedImage =
    previous !== null && (previous.width !== next.width || previous.height !== next.height);
  if (cropState.value.value === null || changedImage) {
    viewCenter.value = null;
    setCrop(
      changedImage || defaultValue === undefined
        ? centeredCrop(bounds.value, constraints.value, autoCropArea)
        : defaultValue,
      "init",
    );
  }
});

function currentImage(): HTMLImageElement | null {
  const image = element.value?.querySelector('[data-vize-ui="image-cropper-image"]');
  return image instanceof HTMLImageElement ? image : null;
}

async function toBlob(options: Omit<CropImageOptions, "rotation"> = {}): Promise<Blob> {
  const image = currentImage();
  if (image === null || crop.value === null) {
    throw new ImageCropperError("VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD", "no loaded image to crop");
  }
  return await cropImage(image, crop.value, { ...options, rotation: rotationValue.value });
}

async function toDataUrl(options: Omit<CropImageOptions, "rotation"> = {}): Promise<string> {
  const image = currentImage();
  if (image === null || crop.value === null) {
    throw new ImageCropperError("VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD", "no loaded image to crop");
  }
  return await cropImage(image, crop.value, {
    ...options,
    output: "data-url",
    rotation: rotationValue.value,
  });
}

imageCropperContext.provide({
  bounds,
  center,
  commit,
  crop,
  disabled: computed(() => disabled),
  id: baseId,
  messages: messageSet,
  moveFrom,
  naturalSize,
  nudgeStep: computed(() => (Number.isFinite(nudgeStep) && nudgeStep > 0 ? nudgeStep : 1)),
  panTo(next) {
    viewCenter.value = next;
  },
  ready,
  resizeFrom,
  rotateBy,
  rotation: rotationValue,
  scale,
  setCrop,
  setInteraction(next) {
    interaction.value = next;
  },
  setNaturalSize(size) {
    naturalSize.value = size;
  },
  setViewportSize(size) {
    if (size.width !== viewportSize.value.width || size.height !== viewportSize.value.height) {
      viewportSize.value = size;
    }
  },
  slotState,
  viewportSize,
  zoom: zoomValue,
  zoomBy,
  zoomTo: (next, anchor) => !disabled && zoomTo(next, anchor),
} satisfies ImageCropperContextValue);

type ImageCropperRootSetupExpose = Omit<
  ImageCropperRootExpose,
  keyof ImageCropperSlotState | "element"
> & {
  readonly crop: ComputedRef<CropArea | null>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly interaction: typeof interaction;
  readonly naturalSize: typeof naturalSize;
  readonly ready: ComputedRef<boolean>;
  readonly rotation: ComputedRef<number>;
  readonly zoom: ComputedRef<number>;
};

const exposed = {
  crop,
  disabled: computed(() => disabled),
  element,
  interaction,
  naturalSize,
  ready,
  reset,
  rotation: rotationValue,
  setCrop: (next: CropArea) => setCrop(next, "api"),
  setRotation,
  setZoom: (next: number) => zoomTo(next),
  toBlob,
  toDataUrl,
  zoom: zoomValue,
} satisfies ImageCropperRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    data-vize-ui="image-cropper-root"
    part="root"
    :data-ready="ready ? 'true' : undefined"
    :data-interaction="interaction"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
