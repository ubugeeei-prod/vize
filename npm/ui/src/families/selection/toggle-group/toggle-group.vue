<script setup lang="ts">
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { toggleGroupContext } from "./toggle-group-context.ts";
import type {
  ToggleGroupContextValue,
  ToggleGroupItemRegistration,
  ToggleGroupNavigationIntent,
} from "./toggle-group-context.ts";
import type {
  ToggleGroupExpose,
  ToggleGroupItemState,
  ToggleGroupOrientation,
  ToggleGroupSlotState,
  ToggleGroupState,
  ToggleGroupType,
  ToggleGroupValue,
} from "./toggle-group-types.ts";
import {
  getNextToggleGroupValue,
  getToggleGroupPressedValues,
  hasToggleGroupValue,
  normalizeToggleGroupValue,
  toggleGroupValueEquals,
} from "./toggle-group-value.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const {
  as = "div",
  type = "single",
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  orientation = "horizontal",
  loop = true,
  rovingFocus = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Native element, custom element, or component to render. @default "div" */
  readonly as?: PrimitiveAs;
  /** Selection mode for item activation. @default "single" */
  readonly type?: ToggleGroupType;
  /** Controlled selected value. `undefined` selects uncontrolled behavior. @default undefined */
  readonly modelValue?: ToggleGroupValue;
  /** Initial value for uncontrolled use and the value restored by reset. @default undefined */
  readonly defaultValue?: ToggleGroupValue;
  /** Disable every item and remove the group from roving focus. @default false */
  readonly disabled?: boolean;
  /** Directional layout hint used by arrow-key roving focus. @default "horizontal" */
  readonly orientation?: ToggleGroupOrientation;
  /** Whether arrow-key navigation wraps at the first and last enabled item. @default true */
  readonly loop?: boolean;
  /** Whether items participate in a single-tabstop roving focus model. @default true */
  readonly rovingFocus?: boolean;
  /** Accessible name when no visible label or `aria-labelledby` supplies one. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label the group. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe the group. @default undefined */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Compound ToggleGroup items. Receives normalized selection and navigation state. */
  default(props: ToggleGroupSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the selected value requests a new controlled value. */
  "update:modelValue": [value: ToggleGroupValue];
  /** Fired after user activation requests a distinct group value. */
  change: [value: ToggleGroupValue, previous: ToggleGroupValue, nativeEvent: MouseEvent];
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const items = new Set<ToggleGroupItemRegistration>();
const typeState = computed(() => type);
const disabledState = computed(() => disabled);
const orientationState = computed(() => orientation);
const rovingFocusState = computed(() => rovingFocus);
const valueState = useControllableState<ToggleGroupValue>({
  value: () =>
    modelValue === undefined ? undefined : normalizeToggleGroupValue(modelValue, typeState.value),
  defaultValue: () => normalizeToggleGroupValue(defaultValue, typeState.value),
  equals: (left, right) => toggleGroupValueEquals(left, right, typeState.value),
  onChange: (value) => emit("update:modelValue", value),
});
const value = computed(() => normalizeToggleGroupValue(valueState.value.value, typeState.value));
const pressedValues = computed(() => getToggleGroupPressedValues(value.value, typeState.value));
const activeValue = shallowRef<string | null>(pressedValues.value[0] ?? null);
const state = computed<ToggleGroupState>(() => {
  if (disabledState.value) return "disabled";
  return pressedValues.value.length === 0 ? "empty" : "selected";
});
const dataValue = computed(() =>
  pressedValues.value.length === 0 ? undefined : pressedValues.value.join(" "),
);
const slotState = computed<ToggleGroupSlotState>(() => ({
  disabled: disabledState.value,
  orientation: orientationState.value,
  pressedValues: pressedValues.value,
  state: state.value,
  type: typeState.value,
  value: value.value,
}));

watch(
  element,
  (root, _previous, onCleanup) => {
    const form = getElementNode(root)?.closest("form");
    if (form == null) return;
    const onReset = () => {
      if (!valueState.controlled.value) valueState.reset();
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);
watch([disabledState, orientationState, pressedValues, rovingFocusState], () => syncActiveValue(), {
  flush: "post",
  immediate: true,
});

function getElementNode(target: PrimitiveElement | null): Element | null {
  if (target instanceof Element) return target;
  if (target != null && target.$el instanceof Element) return target.$el;
  return null;
}

function focusTarget(target: PrimitiveElement | null, options?: FocusOptions): boolean {
  const node = getElementNode(target);
  if (node instanceof HTMLElement) {
    node.focus(options);
    return true;
  }
  if (target != null && "focus" in target && typeof target.focus === "function") {
    target.focus(options);
    return true;
  }
  return false;
}

function getOrderedItems(): readonly ToggleGroupItemRegistration[] {
  const enabled = [...items].filter((item) => !item.disabled.value);
  const root = getElementNode(element.value);
  if (root == null) return enabled;

  const positions = new Map<Element, number>();
  root
    .querySelectorAll('[data-vize-ui="toggle-group-item"]')
    .forEach((node, index) => positions.set(node, index));

  return enabled.sort((left, right) => {
    const leftOrder = positions.get(getElementNode(left.element.value) ?? root) ?? enabled.length;
    const rightOrder = positions.get(getElementNode(right.element.value) ?? root) ?? enabled.length;
    return leftOrder - rightOrder;
  });
}

function syncActiveValue(): void {
  if (disabledState.value || !rovingFocusState.value) {
    activeValue.value = null;
    return;
  }

  const enabled = getOrderedItems();
  const activeItem = enabled.find((item) => item.value() === activeValue.value);
  const activeIsPendingPressed =
    activeValue.value !== null && pressedValues.value.includes(activeValue.value);
  if (activeItem !== undefined || activeIsPendingPressed) return;

  activeValue.value =
    pressedValues.value.find((selected) => enabled.some((item) => item.value() === selected)) ??
    enabled[0]?.value() ??
    null;
}

function focusItem(item: ToggleGroupItemRegistration, options?: FocusOptions): void {
  activeValue.value = item.value();
  focusTarget(item.element.value, options);
}

function focus(options?: FocusOptions): void {
  syncActiveValue();
  const enabled = getOrderedItems();
  const target =
    pressedValues.value
      .map((selected) => enabled.find((item) => item.value() === selected))
      .find((item): item is ToggleGroupItemRegistration => item !== undefined) ??
    enabled.find((item) => item.value() === activeValue.value) ??
    enabled[0];
  if (target !== undefined) focusItem(target, options);
}

function moveFocus(
  fromValue: string,
  intent: ToggleGroupNavigationIntent,
  options?: FocusOptions,
): boolean {
  if (disabledState.value || !rovingFocusState.value) return false;
  const enabled = getOrderedItems();
  if (enabled.length === 0) return false;

  const fromIndex = enabled.findIndex((item) => item.value() === fromValue);
  const activeIndex = enabled.findIndex((item) => item.value() === activeValue.value);
  const currentIndex = fromIndex >= 0 ? fromIndex : activeIndex;
  const rawIndex =
    intent === "first"
      ? 0
      : intent === "last"
        ? enabled.length - 1
        : currentIndex < 0
          ? intent === "next"
            ? 0
            : enabled.length - 1
          : currentIndex + (intent === "next" ? 1 : -1);
  const targetIndex = loop ? (rawIndex + enabled.length) % enabled.length : rawIndex;
  const target = enabled[targetIndex];
  if (target === undefined) return false;

  focusItem(target, options);
  return true;
}

function getItemState(itemValue: string, itemDisabled: boolean): ToggleGroupItemState {
  if (itemDisabled) return "disabled";
  return hasToggleGroupValue(value.value, typeState.value, itemValue) ? "pressed" : "unpressed";
}

function getCurrentValue(): ToggleGroupValue {
  return value.value;
}

function requestItemToggle(itemValue: string, nativeEvent: MouseEvent): boolean {
  const previous = getCurrentValue();
  const next = getNextToggleGroupValue(previous, typeState.value, itemValue);
  setActiveValue(itemValue);
  const changed = valueState.set(next);
  if (changed) emit("change", next, previous, nativeEvent);
  return changed;
}

function setValue(next: ToggleGroupValue): boolean {
  const normalized = normalizeToggleGroupValue(next, typeState.value);
  const changed = valueState.set(normalized);
  activeValue.value =
    getToggleGroupPressedValues(normalized, typeState.value)[0] ?? activeValue.value;
  return changed;
}

function toggleValue(itemValue: string, pressed?: boolean): boolean {
  const next = getNextToggleGroupValue(value.value, typeState.value, itemValue, pressed);
  const changed = valueState.set(next);
  activeValue.value = getToggleGroupPressedValues(next, typeState.value)[0] ?? itemValue;
  return changed;
}

function registerItem(item: ToggleGroupItemRegistration): () => void {
  items.add(item);
  syncActiveValue();
  void nextTick(syncActiveValue);
  return () => {
    const removedValue = item.value();
    items.delete(item);
    if (activeValue.value === removedValue) activeValue.value = null;
    syncActiveValue();
  };
}

function setActiveValue(itemValue: string): void {
  if (!disabledState.value) activeValue.value = itemValue;
}

const context = toggleGroupContext.provide({
  activeValue,
  disabled: disabledState,
  getItemState,
  isPressed: (itemValue) => hasToggleGroupValue(value.value, typeState.value, itemValue),
  moveFocus,
  orientation: orientationState,
  pressedValues,
  registerItem,
  requestItemToggle,
  rovingFocus: rovingFocusState,
  setActiveValue,
  state,
  syncActiveValue,
  type: typeState,
  value,
} satisfies ToggleGroupContextValue);

type ToggleGroupSetupExpose = Omit<ToggleGroupExpose, keyof ToggleGroupSlotState | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ToggleGroupOrientation>;
  readonly pressedValues: ComputedRef<readonly string[]>;
  readonly state: ComputedRef<ToggleGroupState>;
  readonly type: ComputedRef<ToggleGroupType>;
  readonly value: ComputedRef<ToggleGroupValue>;
};

const exposed = {
  disabled: disabledState,
  element,
  focus,
  orientation: orientationState,
  pressedValues,
  reset: valueState.reset,
  setValue,
  state,
  toggleValue,
  type: typeState,
  value,
} satisfies ToggleGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="disabledState ? 'true' : undefined"
    data-vize-ui="toggle-group"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-type="typeState"
    :data-value="dataValue"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
