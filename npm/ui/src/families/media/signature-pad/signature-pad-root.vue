<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { signaturePadContext } from "./signature-pad-context.ts";
import type { SignaturePadContextValue } from "./signature-pad-context.ts";
import {
  isSignatureEmpty,
  serializeSignature,
  signatureToDataUrl,
  signatureToSvg,
} from "./signature-pad-path.ts";
import type {
  SignatureDataUrlOptions,
  SignaturePadChangeReason,
  SignaturePadPressureMode,
  SignaturePadRootExpose,
  SignaturePadSlotState,
  SignaturePadState,
  SignaturePadValueFormat,
  SignaturePoint,
  SignatureStroke,
  SignatureSvgOptions,
  SignatureValue,
} from "./signature-pad-types.ts";

const EMPTY: SignatureValue = Object.freeze([]);

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = [],
  disabled = false,
  readOnly = false,
  name = undefined,
  form = undefined,
  valueFormat = "json",
  width = 400,
  height = 200,
  size = 3,
  thinning = 0.6,
  smoothing = 0.5,
  pressure = "auto",
  historyLimit = 100,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled strokes. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: SignatureValue;

  /**
   * Initial strokes for uncontrolled use.
   *
   * @default []
   */
  readonly defaultValue?: SignatureValue;

  /**
   * Suppress drawing and every editing control.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Show the signature while suppressing drawing and editing.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Form field name. When set, a hidden input submits the serialized signature.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the form that owns the hidden input.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Serialization written to the hidden input.
   *
   * @default "json"
   */
  readonly valueFormat?: SignaturePadValueFormat;

  /**
   * Drawing-surface width in viewBox units; stroke coordinates use this space.
   *
   * @default 400
   */
  readonly width?: number;

  /**
   * Drawing-surface height in viewBox units.
   *
   * @default 200
   */
  readonly height?: number;

  /**
   * Stroke diameter at full pressure, in viewBox units.
   *
   * @default 3
   */
  readonly size?: number;

  /**
   * How strongly pressure thins the stroke, from `0` to `1`.
   *
   * @default 0.6
   */
  readonly thinning?: number;

  /**
   * Curve smoothing between samples, from `0` (polyline) to `1`.
   *
   * @default 0.5
   */
  readonly smoothing?: number;

  /**
   * Pressure source: pen pressure with simulated pressure for mouse and touch
   * (`auto`), raw pointer pressure, or always simulated from velocity.
   *
   * @default "auto"
   */
  readonly pressure?: SignaturePadPressureMode;

  /**
   * Maximum undo steps kept.
   *
   * @default 100
   */
  readonly historyLimit?: number;
}>();

const emit = defineEmits<{
  /** Fired when the strokes request a new controlled value. */
  "update:modelValue": [value: SignatureValue];

  /** Fired after every distinct value request, with its cause. */
  change: [value: SignatureValue, previous: SignatureValue, reason: SignaturePadChangeReason];

  /** Fired when a pointer starts a stroke. */
  strokeStart: [point: SignaturePoint];

  /** Fired when a stroke is committed. */
  strokeEnd: [stroke: SignatureStroke];
}>();

defineSlots<{
  /** Canvas, guide, and controls. Receives the signature state. */
  default(props: SignaturePadSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "signature-pad" });
const valueState = useControllableState<SignatureValue>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const value = computed(() => valueState.value.value);
const liveStroke = shallowRef<SignatureStroke | null>(null);
const undoStack = shallowRef<readonly SignatureValue[]>([]);
const redoStack = shallowRef<readonly SignatureValue[]>([]);
const empty = computed(() => isSignatureEmpty(value.value));
const drawing = computed(() => liveStroke.value !== null);
const interactive = computed(() => !disabled && !readOnly);
const state = computed<SignaturePadState>(() => (empty.value ? "empty" : "filled"));
const serialized = computed(() =>
  name === undefined ? "" : serializeSignature(value.value, valueFormat, exportOptions()),
);
const slotState = computed<SignaturePadSlotState>(() => ({
  canRedo: interactive.value && redoStack.value.length > 0,
  canUndo: interactive.value && undoStack.value.length > 0,
  disabled,
  drawing: drawing.value,
  empty: empty.value,
  readOnly,
  state: state.value,
  value: value.value,
}));

function exportOptions(overrides: Partial<SignatureDataUrlOptions> = {}): SignatureDataUrlOptions {
  return { height, size, smoothing, thinning, width, ...overrides };
}

function toSvg(options: Partial<SignatureSvgOptions> = {}): string {
  return signatureToSvg(value.value, exportOptions(options));
}

async function toDataUrl(options: Partial<SignatureDataUrlOptions> = {}): Promise<string> {
  return await signatureToDataUrl(value.value, exportOptions(options));
}

function currentValue(): SignatureValue {
  return value.value;
}

function request(next: SignatureValue, reason: SignaturePadChangeReason): boolean {
  const previous = currentValue();
  if (next === previous) return false;
  valueState.set(next);
  emit("update:modelValue", next);
  emit("change", next, previous, reason);
  return true;
}

function record(next: SignatureValue, reason: SignaturePadChangeReason): boolean {
  const previous = currentValue();
  if (!request(next, reason)) return false;
  const limit = Math.max(0, Math.floor(historyLimit));
  undoStack.value = limit === 0 ? [] : appendStep(undoStack.value, previous).slice(-limit);
  redoStack.value = [];
  return true;
}

function appendStep(
  stack: readonly SignatureValue[],
  step: SignatureValue,
): readonly SignatureValue[] {
  return stack.concat([step]);
}

function appendPoints(stroke: SignatureStroke, points: readonly SignaturePoint[]): SignatureStroke {
  return Object.freeze({ points: Object.freeze(stroke.points.concat(points)) });
}

function appendStroke(strokes: SignatureValue, stroke: SignatureStroke): SignatureValue {
  return Object.freeze(strokes.concat([stroke]));
}

function startStroke(point: SignaturePoint): boolean {
  if (!interactive.value) return false;
  liveStroke.value = Object.freeze({ points: Object.freeze([point]) });
  emit("strokeStart", point);
  return true;
}

function extendStroke(points: readonly SignaturePoint[]): void {
  if (liveStroke.value === null || points.length === 0) return;
  liveStroke.value = appendPoints(liveStroke.value, points);
}

function finishStroke(): void {
  if (liveStroke.value !== null) commitStroke(liveStroke.value);
  liveStroke.value = null;
}

function commitStroke(stroke: SignatureStroke): void {
  if (stroke.points.length === 0 || !interactive.value) return;
  emit("strokeEnd", stroke);
  record(appendStroke(currentValue(), stroke), "stroke");
}

function cancelStroke(): void {
  liveStroke.value = null;
}

function clear(): boolean {
  if (!interactive.value || empty.value) return false;
  return record(EMPTY, "clear");
}

function undo(): boolean {
  const previous = undoStack.value.at(-1);
  if (!interactive.value || previous === undefined) return false;
  undoStack.value = undoStack.value.slice(0, -1);
  redoStack.value = appendStep(redoStack.value, currentValue());
  request(previous, "undo");
  return true;
}

function redo(): boolean {
  const next = redoStack.value.at(-1);
  if (!interactive.value || next === undefined) return false;
  redoStack.value = redoStack.value.slice(0, -1);
  undoStack.value = appendStep(undoStack.value, currentValue());
  request(next, "redo");
  return true;
}

signaturePadContext.provide({
  cancelStroke,
  clear,
  extendStroke,
  finishStroke,
  height: computed(() => height),
  id: baseId,
  interactive,
  liveStroke,
  pressureMode: computed(() => pressure),
  redo,
  slotState,
  startStroke,
  strokeOptions: computed(() => ({ size, smoothing, thinning })),
  undo,
  value,
  width: computed(() => width),
} satisfies SignaturePadContextValue);

type SignaturePadRootSetupExpose = Omit<
  SignaturePadRootExpose,
  keyof SignaturePadSlotState | "element"
> & {
  readonly canRedo: ComputedRef<boolean>;
  readonly canUndo: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly drawing: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly empty: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly state: ComputedRef<SignaturePadState>;
  readonly value: ComputedRef<SignatureValue>;
};

const exposed = {
  canRedo: computed(() => slotState.value.canRedo),
  canUndo: computed(() => slotState.value.canUndo),
  clear,
  disabled: computed(() => disabled),
  drawing,
  element,
  empty,
  readOnly: computed(() => readOnly),
  redo,
  setValue: (next: SignatureValue) => record(next, "api"),
  state,
  toDataUrl,
  toSvg,
  undo,
  value,
} satisfies SignaturePadRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    data-vize-ui="signature-pad-root"
    part="root"
    :data-state="state"
    :data-drawing="drawing ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-readonly="readOnly ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <input
      v-if="name !== undefined"
      type="hidden"
      :name
      :form
      :value="serialized"
      :disabled="disabled ? true : undefined"
      data-vize-ui="signature-pad-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
