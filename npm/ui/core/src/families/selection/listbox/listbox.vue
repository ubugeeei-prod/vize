<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { createCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import type { CompositeNavigationCommand } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../../controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { listboxContext } from "./listbox-context.ts";
import type { ListboxCollectionValue } from "./listbox-context.ts";
import type {
  ListboxDirection,
  ListboxExpose,
  ListboxOrientation,
  ListboxProps,
  ListboxSelectionMode,
  ListboxSlotState,
  ListboxState,
  ListboxValue,
} from "./listbox-types.ts";
import {
  areListboxValuesEqual,
  emptyListboxValue,
  listboxSelectedValues,
  normalizeListboxValue,
  selectListboxValue,
  toggleListboxValue,
} from "./listbox-value.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  required = false,
  selectionMode = "single",
  orientation = "vertical",
  direction = "ltr",
  loop = false,
  typeahead = true,
  typeaheadTimeout = 500,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<ListboxProps>();

defineSlots<{
  /** Compound ListboxItem options. Receives current selection and focus state. */
  default(props: ListboxSlotState): unknown;

  /** Empty collection fallback rendered when no options are registered. */
  empty(props: ListboxSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the selected value requests a new controlled value. */
  "update:modelValue": [value: ListboxValue];

  /** Fired after user selection requests a distinct Listbox value. */
  change: [value: ListboxValue, previous: ListboxValue, nativeEvent: Event];
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const listboxRole = "listbox" as const;
const listboxId = useDeterministicId({ id: () => id, hint: "listbox" });
const disabledState = computed(() => disabled);
const requiredState = computed(() => required);
const selectionModeState = computed(() => selectionMode);
const orientationState = computed(() => orientation);
const directionState = computed(() => direction);
const valueState = useControllableState<ListboxValue>({
  value: () =>
    modelValue === undefined
      ? undefined
      : normalizeListboxValue(modelValue, selectionModeState.value),
  defaultValue: () => normalizeListboxValue(defaultValue, selectionModeState.value),
  equals: areListboxValuesEqual,
  onChange: (value) => emit("update:modelValue", value),
});
const selectedValue = computed(() =>
  normalizeListboxValue(valueState.value.value, selectionModeState.value),
);
const selectedValues = computed(() => listboxSelectedValues(selectedValue.value));
const selectedSet = computed(() => new Set(selectedValues.value));
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const registry = createCollectionRegistry<string, ListboxCollectionValue>();
const navigation = createCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: ({ value }) => value.id.value,
  isDisabled: disabledState,
  orientation: orientationState,
  direction: directionState,
  loop: () => loop,
  typeahead: {
    isDisabled: () => !typeahead,
    timeout: () => typeaheadTimeout,
  },
});
const activeValue = computed(
  () => registry.activeKey.value ?? registry.navigableItems.value[0]?.key ?? null,
);
const state = computed<ListboxState>(() => {
  if (disabledState.value) return "disabled";
  return selectedValues.value.length === 0 ? "empty" : "selected";
});
const slotState = computed<ListboxSlotState>(() => ({
  activeValue: activeValue.value,
  direction: directionState.value,
  disabled: disabledState.value,
  invalid: invalid.value,
  orientation: orientationState.value,
  required: requiredState.value,
  selectedValues: selectedValues.value,
  selectionMode: selectionModeState.value,
  state: state.value,
  value: selectedValue.value,
}));
const containerProps = computed(() => navigation.getContainerProps());
const listboxActiveDescendant = computed(() =>
  disabledState.value ? undefined : containerProps.value["aria-activedescendant"],
);
const listboxTabindex = computed<0 | undefined>(() =>
  disabledState.value ? undefined : containerProps.value.tabindex,
);
const listboxInteractiveProps = computed<{
  readonly role: "listbox";
  readonly tabindex?: 0;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({
  role: listboxRole,
  ...(listboxTabindex.value === undefined ? {} : { tabindex: listboxTabindex.value }),
  onFocus,
  onKeydown,
}));
const selectionCount = computed(() => selectedValues.value.length);
const dataValue = computed(() =>
  selectedValues.value.length === 1 ? selectedValues.value[0] : undefined,
);

watch(
  [selectedSet, registry.navigableItems],
  () => {
    if (disabledState.value || registry.activeKey.value !== null) return;
    const selected = registry.navigableItems.value.find((item) => selectedSet.value.has(item.key));
    if (selected !== undefined) registry.setActiveKey(selected.key);
  },
  { flush: "sync", immediate: true },
);

function commitSelection(next: ListboxValue, nativeEvent: Event | null): boolean {
  const normalized = normalizeListboxValue(next, selectionModeState.value);
  let previous = emptyListboxValue(selectionModeState.value);
  const changed = valueState.set((current) => {
    previous = current;
    return normalized;
  });
  if (changed && nativeEvent !== null) emit("change", normalized, previous, nativeEvent);
  return changed;
}

function setValue(next: ListboxValue): boolean {
  return commitSelection(next, null);
}

function selectValue(next: string, nativeEvent: Event | null = null): boolean {
  return commitSelection(
    selectListboxValue(selectedValue.value, next, selectionModeState.value),
    nativeEvent,
  );
}

function toggleValue(next: string, nativeEvent: Event | null = null): boolean {
  return commitSelection(
    toggleListboxValue(selectedValue.value, next, selectionModeState.value),
    nativeEvent,
  );
}

function clear(): boolean {
  return commitSelection(emptyListboxValue(selectionModeState.value), null);
}

function setActiveValue(next: string | null): boolean {
  if (disabledState.value) return false;
  if (next !== null && !registry.navigableItems.value.some((item) => item.key === next)) {
    return false;
  }
  return registry.setActiveKey(next);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function navigate(
  command: CompositeNavigationCommand,
  nativeEvent: Event | null = null,
): string | null {
  return navigation.navigate(command, nativeEvent);
}

function commitActiveSelection(next: string | null, nativeEvent: Event): boolean {
  if (next === null) return false;
  if (selectionModeState.value === "multiple") toggleValue(next, nativeEvent);
  else selectValue(next, nativeEvent);
  return true;
}

function onFocus(event: FocusEvent): void {
  containerProps.value.onFocus(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (
    !event.defaultPrevented &&
    !disabledState.value &&
    event.target === event.currentTarget &&
    (event.key === "Enter" || event.key === " ")
  ) {
    if (commitActiveSelection(activeValue.value, event)) {
      event.preventDefault();
    }
    return;
  }
  containerProps.value.onKeydown(event);
}

const context = listboxContext.provide({
  activeValue,
  direction: directionState,
  disabled: disabledState,
  focus,
  getItemProps: (value) => navigation.getItemProps(value),
  id: listboxId,
  invalid,
  orientation: orientationState,
  registerItem: ({ value, id, element, textValue, disabled, order }) =>
    registry.register({ key: value, value: { id, value }, element, textValue, disabled, order }),
  required: requiredState,
  selectedValues: selectedSet,
  selectionMode: selectionModeState,
  selectValue,
  setActiveValue,
  state,
  toggleValue,
  value: selectedValue,
});

type ListboxSetupExpose = Omit<
  ListboxExpose,
  | "activeValue"
  | "direction"
  | "disabled"
  | "element"
  | "id"
  | "invalid"
  | "orientation"
  | "required"
  | "selectedValues"
  | "selectionMode"
  | "state"
  | "value"
> & {
  readonly activeValue: ComputedRef<string | null>;
  readonly direction: ComputedRef<ListboxDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly orientation: ComputedRef<ListboxOrientation>;
  readonly required: ComputedRef<boolean>;
  readonly selectedValues: ComputedRef<readonly string[]>;
  readonly selectionMode: ComputedRef<ListboxSelectionMode>;
  readonly state: ComputedRef<ListboxState>;
  readonly value: ComputedRef<ListboxValue>;
};

const exposed = {
  activeValue,
  clear,
  direction: directionState,
  disabled: disabledState,
  element,
  focus,
  id: listboxId,
  invalid,
  navigate,
  orientation: orientationState,
  required: requiredState,
  reset: valueState.reset,
  selectValue,
  selectedValues,
  selectionMode: selectionModeState,
  setActiveValue,
  setValue,
  state,
  toggleValue,
  value: selectedValue,
} satisfies ListboxSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="listboxInteractiveProps"
    :id="context.id.value"
    ref="element"
    :aria-activedescendant="listboxActiveDescendant"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="orientationState"
    :aria-multiselectable="selectionModeState === 'multiple' ? 'true' : undefined"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-disabled="disabledState ? 'true' : undefined"
    :aria-required="requiredState ? 'true' : undefined"
    data-vize-ui="listbox"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-selection-mode="selectionModeState"
    :data-selection-count="selectionCount"
    :data-value="dataValue"
  >
    <slot v-bind="slotState" />
    <slot v-if="registry.items.value.length === 0" name="empty" v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
