<script setup lang="ts">
import { computed, useId, useTemplateRef } from "vue";
import { encode } from "uqr";

import { createQRPath } from "./qr-path.ts";
import type { QRCodeValue, QRErrorCorrection } from "./qr-path.ts";

const {
  value,
  label,
  size = 160,
  margin = 4,
  errorCorrection = "M",
  foreground = "currentColor",
  background = "transparent",
} = defineProps<{
  /** UTF-8 text or raw bytes encoded in the symbol. */
  readonly value: QRCodeValue;

  /** Non-empty accessible name that describes the QR code's purpose. */
  readonly label: string;

  /**
   * Rendered inline size in CSS pixels.
   *
   * @default 160
   */
  readonly size?: number;

  /**
   * Quiet-zone width measured in modules.
   *
   * @default 4
   */
  readonly margin?: number;

  /**
   * Error-correction level.
   *
   * @default "M"
   */
  readonly errorCorrection?: QRErrorCorrection;

  /**
   * Dark module color. CSS color values and custom properties are accepted.
   *
   * @default "currentColor"
   */
  readonly foreground?: string;

  /**
   * Quiet-zone and light module color.
   *
   * @default "transparent"
   */
  readonly background?: string;
}>();

const element = useTemplateRef<SVGSVGElement>("element");
const titleId = useId();
const accessibleLabel = computed(() => requireNonEmptyText(label, "QR label"));
const renderedSize = computed(() => requirePositiveFinite(size, "QR size"));
const quietZone = computed(() => requireNonNegativeSafeInteger(margin, "QR margin"));
const symbol = computed(() =>
  encode(validateValue(value), {
    border: 0,
    ecc: validateErrorCorrection(errorCorrection),
  }),
);
const path = computed(() => createQRPath(symbol.value.data));
const viewSize = computed(() => symbol.value.size + quietZone.value * 2);
const viewBox = computed(
  () => `${-quietZone.value} ${-quietZone.value} ${viewSize.value} ${viewSize.value}`,
);

function validateValue(candidate: QRCodeValue): QRCodeValue {
  if (typeof candidate === "string") {
    if (candidate.length === 0) {
      throw new TypeError("[VIZE_UI_MEDIA_INVALID_QR_VALUE] QR value must not be empty");
    }

    return candidate;
  }

  if (
    !Array.isArray(candidate) ||
    candidate.length === 0 ||
    candidate.some((byte) => !Number.isSafeInteger(byte) || byte < 0 || byte > 255)
  ) {
    throw new RangeError(
      "[VIZE_UI_MEDIA_INVALID_QR_VALUE] QR bytes must contain values from 0 to 255",
    );
  }

  return candidate;
}

function validateErrorCorrection(candidate: QRErrorCorrection): QRErrorCorrection {
  if (candidate === "L" || candidate === "M" || candidate === "Q" || candidate === "H") {
    return candidate;
  }

  throw new RangeError(
    `[VIZE_UI_MEDIA_INVALID_QR_CORRECTION] Unknown error-correction level: ${String(candidate)}`,
  );
}

function requireNonEmptyText(candidate: string, name: string): string {
  const normalized = candidate.trim();
  if (normalized.length === 0) {
    throw new TypeError(`[VIZE_UI_MEDIA_INVALID_QR_TEXT] ${name} must not be empty`);
  }

  return normalized;
}

function requirePositiveFinite(candidate: number, name: string): number {
  if (!Number.isFinite(candidate) || candidate <= 0) {
    throw new RangeError(`[VIZE_UI_MEDIA_INVALID_QR_SIZE] ${name} must be positive and finite`);
  }

  return candidate;
}

function requireNonNegativeSafeInteger(candidate: number, name: string): number {
  if (!Number.isSafeInteger(candidate) || candidate < 0) {
    throw new RangeError(
      `[VIZE_UI_MEDIA_INVALID_QR_MARGIN] ${name} must be a non-negative safe integer`,
    );
  }

  return candidate;
}

defineExpose({ element });
</script>

<template>
  <svg
    ref="element"
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="viewBox"
    :width="renderedSize"
    :height="renderedSize"
    role="img"
    :aria-labelledby="titleId"
    data-vize-ui="qr-code"
    shape-rendering="crispEdges"
  >
    <title :id="titleId">{{ accessibleLabel }}</title>
    <rect :x="-quietZone" :y="-quietZone" :width="viewSize" :height="viewSize" :fill="background" />
    <path :d="path" :fill="foreground" />
  </svg>
</template>
