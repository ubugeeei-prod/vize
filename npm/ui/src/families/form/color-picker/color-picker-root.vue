<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import {
  DEFAULT_COLOR,
  colorEquals,
  formatColor,
  isColorValue,
  parseColor,
  setColorChannelValue,
  toCssColor,
  toHsla,
} from "./color-picker-color.ts";
import { formatColorChannelValue, getColorChannelLabel } from "./color-picker-color.ts";
import type { ColorChannel, ColorFormat, ColorValue } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import type { ColorPickerContextValue } from "./color-picker-context.ts";
import type {
  ColorPickerChangeDetail,
  ColorPickerChangeSource,
  ColorPickerDirection,
  ColorPickerMessages,
  ColorPickerRootExpose,
  ColorPickerSlotState,
  ColorPickerState,
} from "./color-picker-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = "#000000",
  format = "hex",
  disabled = false,
  readOnly = false,
  required = false,
  name = undefined,
  form = undefined,
  dir = "ltr",
  messages = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled color as any CSS color string accepted by `parseColor`. `undefined`
   * selects uncontrolled behavior. Unparseable strings keep the last valid color.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled color, also restored by `reset()` and native form reset.
   *
   * @default "#000000"
   */
  readonly defaultValue?: string;

  /**
   * Serialization used for `update:modelValue`, slot `value`, and the form value.
   *
   * @default "hex"
   */
  readonly format?: ColorFormat;

  /**
   * Remove every part from interaction and sequential focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep parts focusable while rejecting edits.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the form value as required (forwarded to the hidden input's `data-required`).
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Native form field name. When set, a hidden input submits the formatted value.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the root's DOM ancestry.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Reading direction used for horizontal pointer and arrow-key mapping.
   *
   * @default "ltr"
   */
  readonly dir?: ColorPickerDirection;

  /**
   * Localized accessible strings (channel names, value text, area name).
   * Omitted entries use English defaults.
   *
   * @default undefined
   */
  readonly messages?: ColorPickerMessages;
}>();

const emit = defineEmits<{
  /** Fired with the formatted value whenever the color changes. */
  "update:modelValue": [value: string];

  /** Fired after every distinct color change with the normalized color and its source. */
  change: [value: string, detail: ColorPickerChangeDetail];

  /** Fired when a continuous interaction (drag end, key press, field commit, pick) settles. */
  commit: [value: string, color: ColorValue];
}>();

defineSlots<{
  /** Compound ColorPicker parts. Receives the current color and availability state. */
  default(props: ColorPickerSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const hiddenInput = useTemplateRef<HTMLInputElement>("hiddenInput");
const baseId = useDeterministicId({ id: () => id, hint: "color-picker" });
const formatState = computed<ColorFormat>(() => format);
const disabledState = computed(() => disabled);
const readOnlyState = computed(() => readOnly);
const dirState = computed<ColorPickerDirection>(() => dir);
const editable = computed(() => !disabled && !readOnly);
const state = computed<ColorPickerState>(() => {
  if (disabled) return "disabled";
  return readOnly ? "readonly" : "interactive";
});

const valueState = useControllableState<string>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
// The last accepted color keeps full HSB precision (hue survives greys, saturation survives black).
const internal = shallowRef<ColorValue>(parseColor(valueState.value.value) ?? DEFAULT_COLOR);

const color = computed<ColorValue>(() => {
  const parsed = parseColor(valueState.value.value);
  if (parsed === null) return internal.value;
  return formatColor(parsed, formatState.value) === formatColor(internal.value, formatState.value)
    ? internal.value
    : parsed;
});
const value = computed(() => formatColor(color.value, formatState.value));
const rootStyle = computed(() => {
  const hsl = toHsla(color.value);
  return {
    "--vize-ui-color-picker-color": toCssColor(color.value),
    "--vize-ui-color-picker-opaque-color": toCssColor(
      setColorChannelValue(color.value, "alpha", 1),
    ),
    "--vize-ui-color-picker-hue": `hsl(${Math.round(hsl.hue)} 100% 50%)`,
    "--vize-ui-color-picker-alpha": String(Math.round(color.value.alpha * 1000) / 1000),
  };
});
const slotState = computed<ColorPickerSlotState>(() => ({
  color: color.value,
  disabled: disabledState.value,
  format: formatState.value,
  readOnly: readOnlyState.value,
  state: state.value,
  value: value.value,
}));

watch(
  () => valueState.value.value,
  (next) => {
    const parsed = parseColor(next);
    if (parsed !== null && formatColor(parsed, format) !== formatColor(internal.value, format)) {
      internal.value = parsed;
    }
  },
  { flush: "sync" },
);

function readColor(): ColorValue {
  return color.value;
}

function applyColor(
  next: ColorValue,
  source: ColorPickerChangeSource,
  nativeEvent: Event | null,
): boolean {
  const previousColor = readColor();
  if (colorEquals(previousColor, next) && previousColor.hue === next.hue) {
    internal.value = next;
    return false;
  }
  internal.value = next;
  const serialized = formatColor(next, formatState.value);
  const changed = serialized !== value.value;
  valueState.set(serialized);
  if (changed) emit("update:modelValue", serialized);
  emit("change", serialized, { color: next, previousColor, source, nativeEvent });
  return changed;
}

function setColorFromPart(
  next: ColorValue,
  source: ColorPickerChangeSource,
  nativeEvent: Event | null,
): boolean {
  if (!editable.value) return false;
  return applyColor(next, source, nativeEvent);
}

function commit(_source: ColorPickerChangeSource, _nativeEvent: Event | null): void {
  emit("commit", value.value, color.value);
}

function setColor(next: ColorValue | string): boolean {
  const parsed = typeof next === "string" ? parseColor(next) : isColorValue(next) ? next : null;
  if (parsed === null) return false;
  return applyColor(parsed, "api", null);
}

function reset(): boolean {
  return applyColor(parseColor(defaultValue) ?? DEFAULT_COLOR, "form-reset", null);
}

watch(
  hiddenInput,
  (input, _previous, onCleanup) => {
    const owner = input?.form;
    if (owner === undefined || owner === null) return;
    const onReset = (event: Event) => {
      if (!valueState.controlled.value)
        applyColor(parseColor(defaultValue) ?? DEFAULT_COLOR, "form-reset", event);
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function channelLabel(channel: ColorChannel): string {
  return messages?.channelLabel?.(channel) ?? getColorChannelLabel(channel);
}

colorPickerContext.provide({
  areaLabel: (x, y) =>
    messages?.areaLabel?.(channelLabel(x), channelLabel(y)) ??
    `${channelLabel(x)} and ${channelLabel(y)}`,
  channelLabel,
  channelValueText: (channel, channelValue) =>
    messages?.channelValueText?.(channel, channelValue, channelLabel(channel)) ??
    formatColorChannelValue(channel, channelValue, channelLabel(channel)),
  color,
  commit,
  dir: dirState,
  disabled: disabledState,
  editable,
  format: formatState,
  getPartId: (part) => deriveDeterministicId(baseId.value, part),
  id: baseId,
  readOnly: readOnlyState,
  setColor: setColorFromPart,
  state,
  value,
} satisfies ColorPickerContextValue);

type ColorPickerRootSetupExpose = Omit<
  ColorPickerRootExpose,
  keyof ColorPickerSlotState | "element" | "id"
> & {
  readonly color: ComputedRef<ColorValue>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly format: ComputedRef<ColorFormat>;
  readonly id: ComputedRef<string>;
  readonly readOnly: ComputedRef<boolean>;
  readonly state: ComputedRef<ColorPickerState>;
  readonly value: ComputedRef<string>;
};

const exposed = {
  color,
  disabled: disabledState,
  element,
  format: formatState,
  id: baseId,
  readOnly: readOnlyState,
  reset,
  setColor,
  state,
  value,
} satisfies ColorPickerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    role="group"
    :dir="dirState"
    data-vize-ui="color-picker-root"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-readonly="readOnlyState ? 'true' : undefined"
    :data-value="value"
    :style="rootStyle"
  >
    <slot v-bind="slotState" />
    <input
      v-if="name !== undefined"
      ref="hiddenInput"
      type="hidden"
      :name
      :form
      :value
      :disabled="disabledState"
      :data-required="required ? 'true' : undefined"
      data-vize-ui="color-picker-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
