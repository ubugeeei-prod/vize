<script setup lang="ts">
import { computed, nextTick, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import {
  DEFAULT_COLOR,
  colorEquals,
  formatColor,
  parseColor,
  toCssColor,
} from "./color-picker-color.ts";
import type { ColorValue } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import { colorPickerSwatchGroupContext } from "./color-picker-swatch-context.ts";
import type {
  ColorPickerSwatchExpose,
  ColorPickerSwatchSlotState,
  ColorPickerSwatchState,
} from "./color-picker-types.ts";

const {
  value,
  disabled = false,
  label = undefined,
  order = undefined,
} = defineProps<{
  /** CSS color selected by this swatch, e.g. `"#ff0000"` or `"hsl(210 80% 40%)"`. @default required */
  readonly value: string;

  /**
   * Disable this swatch while keeping it visible.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name. Defaults to the color serialized in the root format.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Deterministic order for virtualized or portalled swatches.
   *
   * @default undefined
   */
  readonly order?: number;
}>();

const emit = defineEmits<{
  /** Fired before selection. Call `preventDefault()` to keep the current color. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Swatch contents, e.g. a check mark. Receives selection state. */
  default(props: ColorPickerSwatchSlotState): unknown;
}>();

const context = colorPickerContext.use();
const group = colorPickerSwatchGroupContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const parsed = computed(() => parseColor(value));
const color = computed<ColorValue>(() => parsed.value ?? DEFAULT_COLOR);
const swatchDisabled = computed(() => context.disabled.value || disabled || parsed.value === null);
const checked = computed(
  () => parsed.value !== null && colorEquals(parsed.value, context.color.value),
);
const state = computed<ColorPickerSwatchState>(() => {
  if (swatchDisabled.value) return "disabled";
  return checked.value ? "checked" : "unchecked";
});
const accessibleName = computed(() => label ?? formatColor(color.value, context.format.value));
const navigationProps = computed(() => group.navigation.getItemProps(value));

const swatchStyle = computed(() => ({
  "--vize-ui-color-picker-swatch-color": toCssColor(color.value),
}));
const slotState = computed<ColorPickerSwatchSlotState>(() => ({
  checked: checked.value,
  color: color.value,
  disabled: swatchDisabled.value,
  state: state.value,
  value,
}));
let registration: CollectionRegistration<string> | null = null;

function register(): void {
  registration?.unregister();
  registration = group.registry.register({
    disabled: swatchDisabled,
    element,
    key: value,
    order: () => order,
    textValue: () => accessibleName.value,
    value,
  });
  group.syncActiveValue();
  void nextTick(group.syncActiveValue);
}

watch(() => value, register, { flush: "sync", immediate: true });
onUnmounted(() => {
  registration?.unregister();
  registration = null;
  void nextTick(group.syncActiveValue);
});

function select(event: Event): void {
  if (swatchDisabled.value || !context.editable.value || parsed.value === null) return;
  if (context.setColor(parsed.value, "swatch", event)) context.commit("swatch", event);
}

function onClick(event: MouseEvent): void {
  if (swatchDisabled.value) return;
  emit("click", event);
  if (!event.defaultPrevented) select(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (swatchDisabled.value) return;
  if (event.key === " " || event.key === "Enter") {
    event.preventDefault();
    select(event);
    return;
  }
  group.navigation.getContainerProps().onKeydown(event);
}

function onFocus(event: FocusEvent): void {
  if (!swatchDisabled.value) navigationProps.value.onFocus(event);
}

function onPointerdown(event: PointerEvent): void {
  if (!swatchDisabled.value) navigationProps.value.onPointerdown(event);
}

// Role, roving tabindex, and handlers are bound together; only the active enabled swatch is tabbable.
const radioProps = computed<{
  readonly role: "radio";
  readonly tabindex: -1 | 0;
  readonly onClick: (event: MouseEvent) => void;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
}>(() => ({
  role: "radio",
  tabindex: swatchDisabled.value ? -1 : (navigationProps.value.tabindex ?? -1),
  onClick,
  onFocus,
  onKeydown,
  onPointerdown,
}));

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type ColorPickerSwatchSetupExpose = Omit<
  ColorPickerSwatchExpose,
  keyof ColorPickerSwatchSlotState | "element"
> & {
  readonly checked: ComputedRef<boolean>;
  readonly color: ComputedRef<ColorValue>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly state: ComputedRef<ColorPickerSwatchState>;
  readonly value: string;
};

const exposed = {
  checked,
  color,
  disabled: swatchDisabled,
  element,
  focus,
  state,
  value,
} satisfies ColorPickerSwatchSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="group.getSwatchId(value)"
    ref="element"
    v-bind="radioProps"
    :aria-checked="checked ? 'true' : 'false'"
    :aria-disabled="swatchDisabled ? 'true' : undefined"
    :aria-label="accessibleName"
    data-vize-ui="color-picker-swatch"
    part="swatch"
    :data-state="state"
    :data-value="value"
    :data-disabled="swatchDisabled ? 'true' : undefined"
    :style="swatchStyle"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
