<script setup lang="ts">
import { computed, ref, useTemplateRef, watch } from "vue";

import { hueFromPoint, pointFromHue, stepHue } from "./color-wheel-geometry.ts";
import type { ColorWheelProps, ColorWheelSlotProps } from "./types.ts";

const { modelValue = 0, label = "Hue", disabled = false } = defineProps<ColorWheelProps>();

const emit = defineEmits<{
  /**
   * Fired when the hue changes from the keyboard or the pointer.
   *
   * Carries the next hue in degrees.
   */
  "update:modelValue": [hue: number];
}>();

defineSlots<{
  /** Renders inside the wheel with the current hue and an `hsl()` color. */
  default(props: ColorWheelSlotProps): unknown;
}>();

const host = useTemplateRef<HTMLElement>("host");
const hue = ref(modelValue);
watch(
  () => modelValue,
  (next) => {
    hue.value = next;
  },
);

const marker = computed(() => pointFromHue(hue.value, 36));
const markerX = computed(() => (50 + marker.value.x).toFixed(2));
const markerY = computed(() => (50 + marker.value.y).toFixed(2));
const announced = computed(() => {
  const rounded = Math.round(hue.value);
  if (rounded === 360) return 360;
  return ((rounded % 360) + 360) % 360;
});
const color = computed(() => `hsl(${announced.value} 100% 50%)`);

function setHue(next: number): void {
  if (disabled) return;
  hue.value = next;
  emit("update:modelValue", next);
}

function onKeydown(event: KeyboardEvent): void {
  if (disabled) return;
  const step = event.shiftKey ? 10 : 1;
  let next: number | undefined;
  switch (event.key) {
    case "ArrowRight":
    case "ArrowUp":
      next = stepHue(hue.value, step);
      break;
    case "ArrowLeft":
    case "ArrowDown":
      next = stepHue(hue.value, -step);
      break;
    case "PageUp":
      next = stepHue(hue.value, 10);
      break;
    case "PageDown":
      next = stepHue(hue.value, -10);
      break;
    case "Home":
      next = 0;
      break;
    case "End":
      next = 360;
      break;
    default:
      return;
  }
  event.preventDefault();
  setHue(next);
}

function applyPointer(event: PointerEvent): void {
  if (disabled) return;
  const current = event.currentTarget;
  if (!(current instanceof HTMLElement)) return;
  const rect = current.getBoundingClientRect();
  const dx = event.clientX - (rect.left + rect.width / 2);
  const dy = event.clientY - (rect.top + rect.height / 2);
  setHue(hueFromPoint(dx, dy));
}

function onPointerDown(event: PointerEvent): void {
  if (disabled) return;
  const current = event.currentTarget;
  if (current instanceof HTMLElement && typeof current.setPointerCapture === "function") {
    current.setPointerCapture(event.pointerId);
  }
  applyPointer(event);
}

function onPointerMove(event: PointerEvent): void {
  const current = event.currentTarget;
  if (!(current instanceof HTMLElement) || typeof current.hasPointerCapture !== "function") return;
  if (!current.hasPointerCapture(event.pointerId)) return;
  applyPointer(event);
}

/** Move focus to the slider. */
function focus(options?: FocusOptions): void {
  host.value?.focus(options);
}

defineExpose({ focus });
</script>

<template>
  <div
    ref="host"
    role="slider"
    data-vize-ui="color-wheel"
    :aria-label="label"
    aria-valuemin="0"
    aria-valuemax="360"
    :aria-valuenow="announced"
    :aria-valuetext="`${announced} degrees`"
    :aria-disabled="disabled ? 'true' : undefined"
    :tabindex="disabled ? -1 : 0"
    :style="{ '--vize-color-wheel-hue': String(hue) }"
    @keydown="onKeydown"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
  >
    <svg viewBox="0 0 100 100" aria-hidden="true">
      <circle cx="50" cy="50" r="36" />
      <circle data-part="marker" :cx="markerX" :cy="markerY" r="4" />
    </svg>
    <slot :hue :color />
  </div>
</template>

<style scoped>
/* Headless by design. Paint the wheel from --vize-color-wheel-hue. */
</style>
