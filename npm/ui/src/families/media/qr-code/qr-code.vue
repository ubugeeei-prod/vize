<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { QrCodeEncodeError, encodeQrCode } from "./qr-code-encoder.ts";
import { normalizeQrCodeQuietZone, qrCodeToSvgPath } from "./qr-code-svg.ts";
import type {
  QrCodeEncodeErrorLike,
  QrCodeErrorCorrection,
  QrCodeKanjiEncoder,
  QrCodeExpose,
  QrCodeMask,
  QrCodeMatrix,
  QrCodeMode,
  QrCodeSegmentation,
  QrCodeSlotState,
  QrCodeState,
  QrCodeValue,
  QrCodeVersion,
} from "./qr-code-types.ts";

const {
  value,
  errorCorrection = "M",
  boostErrorCorrection = false,
  version = "auto",
  mask = "auto",
  mode = "auto",
  segmentation = "optimal",
  kanji = undefined,
  eci = undefined,
  utf8Eci = false,
  quietZone = 4,
  label = undefined,
  decorative = false,
  foreground = "currentColor",
  background = "none",
} = defineProps<{
  /**
   * Text (segmented automatically; byte runs are UTF-8), bytes, or explicit
   * segments from the `createQrCode*Segment` helpers. @default required
   */
  readonly value: QrCodeValue;

  /**
   * Minimum error correction level. Prefer `"H"` when an overlay covers the center.
   *
   * @default "M"
   */
  readonly errorCorrection?: QrCodeErrorCorrection;

  /**
   * Raise the error correction level while the data still fits the chosen version.
   *
   * @default false
   */
  readonly boostErrorCorrection?: boolean;

  /**
   * Fixed symbol version, or `"auto"` for the smallest fitting version.
   *
   * @default "auto"
   */
  readonly version?: QrCodeVersion | "auto";

  /**
   * Fixed data mask, or `"auto"` for the lowest-penalty mask.
   *
   * @default "auto"
   */
  readonly mask?: QrCodeMask | "auto";

  /**
   * Forced single encoding mode, or `"auto"` to follow {@link segmentation}.
   *
   * @default "auto"
   */
  readonly mode?: QrCodeMode | "auto";

  /**
   * Automatic segmentation: `"optimal"` mixes modes for the fewest bits,
   * `"single"` keeps the most compact single mode.
   *
   * @default "optimal"
   */
  readonly segmentation?: QrCodeSegmentation;

  /**
   * Shift_JIS mapping enabling Kanji segments, e.g. `qrCodeKanjiEncoder`
   * imported from the opt-in Kanji table module.
   *
   * @default undefined
   */
  readonly kanji?: QrCodeKanjiEncoder;

  /**
   * ECI designator written before the data (`26` = UTF-8, `20` = Shift_JIS).
   *
   * @default undefined
   */
  readonly eci?: number;

  /**
   * Declare UTF-8 byte data with ECI 26.
   *
   * @default false
   */
  readonly utf8Eci?: boolean;

  /**
   * Light margin in modules around the symbol. Scanners expect 4.
   *
   * @default 4
   */
  readonly quietZone?: number;

  /**
   * Accessible name. Defaults to the text value; byte values need an explicit label.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Hide the symbol from assistive technology when adjacent text already conveys it.
   *
   * @default false
   */
  readonly decorative?: boolean;

  /**
   * SVG fill for dark modules. Consumer CSS on `[part="modules"]` can override it.
   *
   * @default "currentColor"
   */
  readonly foreground?: string;

  /**
   * SVG fill for the light background. Consumer CSS on `[part="background"]` can override it.
   *
   * @default "none"
   */
  readonly background?: string;
}>();

defineSlots<{
  /**
   * SVG content drawn above the modules, in module coordinates (for example a
   * centered `<image>` logo). Only rendered while the value encodes.
   */
  overlay(props: QrCodeSlotState): unknown;

  /** Content rendered instead of the symbol when the value cannot be encoded. */
  fallback(props: QrCodeSlotState): unknown;
}>();

interface QrCodeEncoding {
  readonly matrix: QrCodeMatrix | null;
  readonly error: QrCodeEncodeError | null;
  readonly quietZone: number;
}

const element = useTemplateRef<SVGSVGElement>("element");
const encoding = computed<QrCodeEncoding>(() => {
  try {
    const zone = normalizeQrCodeQuietZone(quietZone);
    const matrix = encodeQrCode(value, {
      boostErrorCorrection,
      eci,
      errorCorrection,
      kanji,
      mask,
      mode,
      segmentation,
      utf8Eci,
      version,
    });
    return { matrix, error: null, quietZone: zone };
  } catch (error) {
    if (error instanceof QrCodeEncodeError) return { matrix: null, error, quietZone: 0 };
    throw error;
  }
});
const matrix = computed(() => encoding.value.matrix);
const error = computed<QrCodeEncodeErrorLike | null>(() => encoding.value.error);
const zone = computed(() => encoding.value.quietZone);
const state = computed<QrCodeState>(() => (matrix.value === null ? "error" : "ready"));
const dimension = computed(() => (matrix.value === null ? 0 : matrix.value.size + zone.value * 2));
const path = computed(() =>
  matrix.value === null ? "" : qrCodeToSvgPath(matrix.value, { quietZone: zone.value }),
);
const versionAttribute = computed<number | undefined>(() => matrix.value?.version);
const errorCorrectionAttribute = computed<string | undefined>(() => matrix.value?.errorCorrection);
const maskAttribute = computed<number | undefined>(() => matrix.value?.mask);
const modeAttribute = computed<string | undefined>(() => matrix.value?.mode ?? undefined);
const sizeAttribute = computed<number | undefined>(() => matrix.value?.size);
const errorCode = computed<string | undefined>(() => error.value?.code);
const accessibleName = computed(() => {
  if (decorative) return undefined;
  if (label !== undefined) return label;
  return typeof value === "string" ? value : undefined;
});
const slotState = computed<QrCodeSlotState>(() => ({
  dimension: dimension.value,
  error: error.value,
  matrix: matrix.value,
  quietZone: zone.value,
  state: state.value,
}));

type QrCodeSetupExpose = Omit<QrCodeExpose, keyof QrCodeSlotState | "element"> & {
  readonly dimension: ComputedRef<number>;
  readonly element: typeof element;
  readonly error: ComputedRef<QrCodeEncodeErrorLike | null>;
  readonly matrix: ComputedRef<QrCodeMatrix | null>;
  readonly quietZone: ComputedRef<number>;
  readonly state: ComputedRef<QrCodeState>;
};

const exposed = {
  dimension,
  element,
  error,
  matrix,
  quietZone: zone,
  state,
} satisfies QrCodeSetupExpose;

defineExpose(exposed);
</script>

<template>
  <svg
    v-if="matrix !== null"
    ref="element"
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="`0 0 ${dimension} ${dimension}`"
    shape-rendering="crispEdges"
    :role="decorative ? undefined : 'img'"
    :aria-label="accessibleName"
    :aria-hidden="decorative ? 'true' : undefined"
    data-vize-ui="qr-code"
    part="root"
    data-state="ready"
    :data-version="versionAttribute"
    :data-error-correction="errorCorrectionAttribute"
    :data-mask="maskAttribute"
    :data-mode="modeAttribute"
    :data-size="sizeAttribute"
  >
    <title v-if="accessibleName !== undefined">{{ accessibleName }}</title>
    <rect
      part="background"
      data-vize-ui="qr-code-background"
      :width="dimension"
      :height="dimension"
      :fill="background"
    />
    <path part="modules" data-vize-ui="qr-code-modules" :d="path" :fill="foreground" />
    <slot name="overlay" v-bind="slotState" />
  </svg>
  <span v-else data-vize-ui="qr-code" part="root" data-state="error" :data-error="errorCode">
    <slot name="fallback" v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
