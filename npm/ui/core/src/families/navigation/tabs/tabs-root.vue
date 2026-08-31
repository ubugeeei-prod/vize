<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../../controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { tabsContext } from "./tabs-context.ts";
import type { TabsContextValue, TabsTriggerRegistrationInput } from "./tabs-context.ts";
import type {
  TabsActivationMode,
  TabsDirection,
  TabsOrientation,
  TabsRootExpose,
  TabsSlotState,
  TabsState,
  TabsTriggerState,
  TabsValue,
} from "./tabs-types.ts";
import { getTabsValueIdSegment, tabsValueEquals } from "./tabs-value.ts";

type TabsInternalValue = TabsValue | undefined;

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  activationMode = "automatic",
  orientation = "horizontal",
  dir = "ltr",
  loop = true,
} = defineProps<{
  /**
   * Consumer-owned Tabs base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled selected tab value. `undefined` selects uncontrolled behavior; `null` clears it.
   *
   * @default undefined
   */
  readonly modelValue?: TabsValue;

  /**
   * Initial selected value for uncontrolled use. `undefined` selects the first enabled trigger.
   *
   * @default undefined
   */
  readonly defaultValue?: TabsValue;

  /**
   * Disable every trigger while preserving the current selected panel.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Whether arrow focus activates tabs immediately or waits for Enter, Space, or click.
   *
   * @default "automatic"
   */
  readonly activationMode?: TabsActivationMode;

  /**
   * Directional layout hint used by roving arrow-key focus.
   *
   * @default "horizontal"
   */
  readonly orientation?: TabsOrientation;

  /**
   * Reading direction used for horizontal arrow-key navigation.
   *
   * @default "ltr"
   */
  readonly dir?: TabsDirection;

  /**
   * Whether arrow-key navigation wraps at the first and last enabled trigger.
   *
   * @default true
   */
  readonly loop?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the selected value requests a new controlled value. */
  "update:modelValue": [value: TabsValue];

  /** Fired after any distinct selected-value request. */
  change: [value: TabsValue, previous: TabsValue, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Tabs children. Receives current selection, orientation, and availability state. */
  default(props: TabsSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "tabs" });
const listId = computed(() => deriveDeterministicId(baseId.value, "list"));
const disabledState = computed(() => disabled);
const activationModeState = computed(() => activationMode);
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const registry = createCollectionRegistry<string, string>({ disabledBehavior: "skip" });
const firstEnabledValue = computed<TabsValue>(() => registry.navigableItems.value[0]?.key ?? null);
const valueState = useControllableState<TabsInternalValue>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  equals: Object.is,
});
const selectedValue = computed<TabsValue>(() => {
  const configured = valueState.value.value;
  if (valueState.controlled.value) return configured ?? null;
  return configured === undefined ? firstEnabledValue.value : configured;
});
const state = computed<TabsState>(() => {
  if (disabledState.value) return "disabled";
  return selectedValue.value === null ? "empty" : "selected";
});
const slotState = computed<TabsSlotState>(() => ({
  activationMode: activationModeState.value,
  dir: dirState.value,
  disabled: disabledState.value,
  orientation: orientationState.value,
  state: state.value,
  value: selectedValue.value,
}));

function getSelectedValue(): TabsValue {
  return selectedValue.value;
}

function getTriggerId(value: string): string {
  return deriveDeterministicId(baseId.value, `trigger-${getTabsValueIdSegment(value)}`);
}

function getContentId(value: string): string {
  return deriveDeterministicId(baseId.value, `content-${getTabsValueIdSegment(value)}`);
}

function setValue(value: TabsValue, nativeEvent: Event | null = null): boolean {
  const previous = getSelectedValue();
  if (tabsValueEquals(previous, value)) return false;
  valueState.set(value);
  if (value !== null && registry.getItem(value)?.disabled === false) registry.setActiveKey(value);
  emit("update:modelValue", value);
  emit("change", value, previous, nativeEvent);
  return true;
}

function reset(): boolean {
  return setValue(defaultValue === undefined ? firstEnabledValue.value : defaultValue, null);
}

function syncActiveValue(): void {
  if (disabledState.value) {
    if (registry.activeKey.value !== null) registry.setActiveKey(null);
    return;
  }
  const selected = getSelectedValue();
  const selectedItem = selected === null ? undefined : registry.getItem(selected);
  const target =
    selectedItem !== undefined && !selectedItem.disabled ? selected : firstEnabledValue.value;
  if (target !== registry.activeKey.value) registry.setActiveKey(target);
}

function focus(options?: FocusOptions): void {
  syncActiveValue();
  const target = registry.activeKey.value ?? firstEnabledValue.value;
  const item = target === null ? undefined : registry.getItem(target);
  if (item?.element instanceof HTMLElement) item.element.focus(options);
}

function getTriggerState(value: string, triggerDisabled: boolean): TabsTriggerState {
  if (triggerDisabled) return "disabled";
  return selectedValue.value === value ? "active" : "inactive";
}

function registerTrigger(input: TabsTriggerRegistrationInput) {
  const registration = registry.register({
    key: input.value,
    value: input.value,
    element: input.element,
    disabled: input.disabled,
    textValue: input.textValue,
    order: input.order,
  });
  syncActiveValue();
  void nextTick(syncActiveValue);
  return registration;
}

const navigation = useCompositeNavigation<string, string>({
  registry,
  focusStrategy: "roving",
  getItemId: ({ key }) => getTriggerId(key),
  orientation: orientationState,
  direction: dirState,
  loop: () => loop,
  isDisabled: disabledState,
  onNavigate(change) {
    if (activationModeState.value === "automatic" && change.intent !== "pointer") {
      setValue(change.key, change.originalEvent);
    }
  },
});

watch([disabledState, selectedValue, registry.navigableItems], () => syncActiveValue(), {
  flush: "post",
  immediate: true,
});

watch(
  registry.items,
  (items, previousItems) => {
    const selected = getSelectedValue();
    if (valueState.controlled.value || selected === null) return;
    const hadSelected = previousItems.some((item) => item.key === selected);
    const hasSelected = items.some((item) => item.key === selected);
    if (hadSelected && !hasSelected) setValue(firstEnabledValue.value, null);
  },
  { flush: "post" },
);

const context = tabsContext.provide({
  activationMode: activationModeState,
  dir: dirState,
  disabled: disabledState,
  focus,
  getContentId,
  getTriggerId,
  getTriggerState,
  id: baseId,
  isSelected: (value) => selectedValue.value === value,
  listId,
  navigation,
  orientation: orientationState,
  registerTrigger,
  registry,
  selectValue: (value, nativeEvent = null) => setValue(value, nativeEvent),
  setValue,
  state,
  syncActiveValue,
  value: selectedValue,
} satisfies TabsContextValue);

type TabsRootSetupExpose = Omit<
  TabsRootExpose,
  keyof TabsSlotState | "element" | "id" | "listId"
> & {
  readonly activationMode: ComputedRef<TabsActivationMode>;
  readonly dir: ComputedRef<TabsDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly orientation: ComputedRef<TabsOrientation>;
  readonly state: ComputedRef<TabsState>;
  readonly value: ComputedRef<TabsValue>;
};

const exposed = {
  activationMode: activationModeState,
  dir: dirState,
  disabled: disabledState,
  element,
  focus: context.focus,
  id: baseId,
  listId,
  orientation: orientationState,
  reset,
  setValue,
  state,
  value: selectedValue,
} satisfies TabsRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :dir="dirState"
    data-vize-ui="tabs-root"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-activation-mode="activationModeState"
    :data-dir="dirState"
    :data-value="selectedValue ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
