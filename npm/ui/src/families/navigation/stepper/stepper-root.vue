<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { stepperContext } from "./stepper-context.ts";
import type {
  StepperCollectionValue,
  StepperContextValue,
  StepperItemRegistrationInput,
} from "./stepper-context.ts";
import type {
  StepperItemState,
  StepperRootProps,
  StepperRootSetupExpose,
  StepperRootState,
  StepperSlotState,
  StepperValue,
} from "./stepper-types.ts";
import {
  getRelativeStepperValue,
  getStepperItemState as resolveStepperItemState,
  isStepperSelectable,
} from "./stepper-navigation.ts";
import { getStepperValueIdSegment, stepperValueEquals } from "./stepper-value.ts";

type StepperInternalValue = StepperValue | undefined;

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  navigationMode = "linear",
  orientation = "horizontal",
  dir = "ltr",
  loop = false,
} = defineProps<StepperRootProps>();

const emit = defineEmits<{
  /** Fired when the current step requests a new controlled value. */
  "update:modelValue": [value: StepperValue];

  /** Fired after any distinct current-step request. */
  change: [value: StepperValue, previous: StepperValue, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Stepper children. Receives current value, completion, and navigation state. */
  default(props: StepperSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "stepper" });
const listId = computed(() => deriveDeterministicId(baseId.value, "list"));
const disabledState = computed(() => disabled);
const navigationModeState = computed(() => navigationMode);
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const registry = createCollectionRegistry<string, StepperCollectionValue>({
  disabledBehavior: "skip",
});
const firstEnabledValue = computed<StepperValue>(
  () => registry.navigableItems.value[0]?.key ?? null,
);
const valueState = useControllableState<StepperInternalValue>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  equals: Object.is,
});
const currentValue = computed<StepperValue>(() => {
  const configured = valueState.value.value;
  if (valueState.controlled.value) return configured ?? null;
  return configured === undefined ? firstEnabledValue.value : configured;
});
const completedValues = computed<readonly string[]>(() =>
  Object.freeze(
    registry.items.value.filter((item) => item.value.completed.value).map((item) => item.key),
  ),
);
const completedCount = computed(() => completedValues.value.length);
const currentIndex = computed(() => {
  const value = getCurrentValue();
  return value === null ? -1 : registry.items.value.findIndex((item) => item.key === value);
});
const state = computed<StepperRootState>(() => {
  if (disabledState.value) return "disabled";
  return currentValue.value === null ? "empty" : "active";
});
const slotState = computed<StepperSlotState>(() => ({
  completedValues: completedValues.value,
  currentIndex: currentIndex.value,
  dir: dirState.value,
  disabled: disabledState.value,
  navigationMode: navigationModeState.value,
  orientation: orientationState.value,
  state: state.value,
  value: currentValue.value,
}));

function getItemId(value: string): string {
  return deriveDeterministicId(baseId.value, `item-${getStepperValueIdSegment(value)}`);
}

function getTriggerId(value: string): string {
  return deriveDeterministicId(baseId.value, `trigger-${getStepperValueIdSegment(value)}`);
}

function getContentId(value: string): string {
  return deriveDeterministicId(baseId.value, `content-${getStepperValueIdSegment(value)}`);
}

function getCurrentValue(): StepperValue {
  return currentValue.value;
}

function isCurrent(value: string): boolean {
  return getCurrentValue() === value;
}

function isCompleted(value: string): boolean {
  return registry.getItem(value)?.value.completed.value === true;
}

function isStepDisabled(value: string): boolean {
  return disabledState.value || registry.getItem(value)?.disabled === true;
}

function getItemIndex(value: string): number {
  return registry.items.value.findIndex((item) => item.key === value);
}

function getItemState(value: string): StepperItemState {
  return resolveStepperItemState({
    completed: isCompleted(value),
    current: isCurrent(value),
    disabled: isStepDisabled(value),
  });
}

function isSelectable(value: string): boolean {
  return isStepperSelectable({
    disabled: disabledState.value,
    item: registry.getItem(value),
    items: registry.items.value,
    navigationMode: navigationModeState.value,
    selected: getCurrentValue(),
    value,
  });
}

function commitValue(value: StepperValue, nativeEvent: Event | null): boolean {
  const previous = getCurrentValue();
  if (stepperValueEquals(previous, value)) return false;
  valueState.set(value);
  if (value !== null && registry.getItem(value)?.disabled === false) registry.setActiveKey(value);
  emit("update:modelValue", value);
  emit("change", value, previous, nativeEvent);
  return true;
}

function setValue(value: StepperValue, nativeEvent: Event | null = null): boolean {
  if (value !== null && !isSelectable(value)) return false;
  return commitValue(value, nativeEvent);
}

function selectValue(value: string, nativeEvent: Event | null = null): boolean {
  return setValue(value, nativeEvent);
}

function selectRelative(direction: "next" | "previous", nativeEvent: Event | null): boolean {
  const target = getRelativeStepperValue(
    registry.navigableItems.value,
    getCurrentValue(),
    direction,
  );
  return target === null ? false : setValue(target, nativeEvent);
}

function next(nativeEvent: Event | null = null): boolean {
  return selectRelative("next", nativeEvent);
}

function previous(nativeEvent: Event | null = null): boolean {
  return selectRelative("previous", nativeEvent);
}

function reset(): boolean {
  return commitValue(defaultValue === undefined ? firstEnabledValue.value : defaultValue, null);
}

function syncActiveValue(): void {
  if (disabledState.value) {
    if (registry.activeKey.value !== null) registry.setActiveKey(null);
    return;
  }
  const selected = getCurrentValue();
  const selectedItem = selected === null ? undefined : registry.getItem(selected);
  const target =
    selectedItem !== undefined && !selectedItem.disabled ? selected : firstEnabledValue.value;
  if (target !== registry.activeKey.value) registry.setActiveKey(target);
}

function focus(options?: FocusOptions): void {
  syncActiveValue();
  const target = registry.activeKey.value ?? firstEnabledValue.value;
  if (target !== null) focusValue(target, options);
}

function focusValue(value: string, options?: FocusOptions): boolean {
  if (disabledState.value) return false;
  const item = registry.getItem(value);
  if (item === undefined || item.disabled || !(item.element instanceof HTMLElement)) return false;
  registry.setActiveKey(value);
  item.element.focus(options);
  return true;
}

function registerItem(input: StepperItemRegistrationInput) {
  const registration = registry.register({
    key: input.value,
    value: {
      completed: input.completed,
      contentId: input.contentId,
      id: input.id,
      triggerId: input.triggerId,
    },
    element: input.element,
    disabled: input.disabled,
    textValue: input.textValue,
    order: input.order,
  });
  syncActiveValue();
  void nextTick(syncActiveValue);
  return registration;
}

const navigation = useCompositeNavigation({
  registry,
  focusStrategy: "roving",
  getItemId: ({ key }) => getTriggerId(key),
  orientation: orientationState,
  direction: dirState,
  loop: () => loop,
  isDisabled: disabledState,
});

watch([disabledState, currentValue, registry.navigableItems], () => syncActiveValue(), {
  flush: "post",
  immediate: true,
});

watch(
  registry.items,
  (items, previousItems) => {
    const selected = getCurrentValue();
    if (valueState.controlled.value || selected === null) return;
    const hadSelected = previousItems.some((item) => item.key === selected);
    const hasSelected = items.some((item) => item.key === selected);
    if (hadSelected && !hasSelected) setValue(firstEnabledValue.value, null);
  },
  { flush: "post" },
);

const context = stepperContext.provide({
  completedValues,
  currentIndex,
  dir: dirState,
  disabled: disabledState,
  focus,
  focusValue,
  getContentId,
  getItemId,
  getItemIndex,
  getItemState,
  getTriggerId,
  id: baseId,
  isCompleted,
  isCurrent,
  isSelectable,
  isStepDisabled,
  listId,
  navigation,
  navigationMode: navigationModeState,
  orientation: orientationState,
  registerItem,
  registry,
  selectValue,
  setValue,
  state,
  syncActiveValue,
  value: currentValue,
} satisfies StepperContextValue);

const exposed = {
  completedValues,
  currentIndex,
  dir: dirState,
  disabled: disabledState,
  element,
  focus,
  id: baseId,
  isSelectable,
  listId,
  navigationMode: navigationModeState,
  next,
  orientation: orientationState,
  previous,
  reset,
  selectValue,
  setValue,
  state,
  value: currentValue,
} satisfies StepperRootSetupExpose<typeof element>;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :dir="dirState"
    data-vize-ui="stepper-root"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-navigation-mode="navigationModeState"
    :data-linear="navigationModeState === 'linear' ? 'true' : undefined"
    :data-dir="dirState"
    :data-value="currentValue ?? undefined"
    :data-current-index="currentIndex"
    :data-completed-count="completedCount"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
