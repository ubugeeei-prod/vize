<script setup lang="ts">
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";
import { buttonGroupContext } from "./button-group-context.ts";
import type {
  ButtonGroupContextValue,
  ButtonGroupItemRegistration,
  ButtonGroupNavigationIntent,
} from "./button-group-context.ts";
import type {
  ButtonGroupExpose,
  ButtonGroupItemState,
  ButtonGroupOrientation,
  ButtonGroupRole,
  ButtonGroupSlotState,
  ButtonGroupState,
} from "./button-group-types.ts";

const {
  as = "div",
  role = "group",
  disabled = false,
  orientation = "horizontal",
  loop = true,
  rovingFocus = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Native element, custom element, or component to render. @default "div" */
  readonly as?: PrimitiveAs;
  /** Accessible grouping semantics for adjacent actions. @default "group" */
  readonly role?: ButtonGroupRole;
  /** Disable every item and remove the group from roving focus. @default false */
  readonly disabled?: boolean;
  /** Directional layout hint used by toolbar arrow-key roving focus. @default "horizontal" */
  readonly orientation?: ButtonGroupOrientation;
  /** Whether arrow-key navigation wraps at the first and last enabled item. @default true */
  readonly loop?: boolean;
  /**
   * Whether items participate in a single-tabstop roving focus model.
   * Defaults to true for `role="toolbar"` and false for plain groups.
   *
   * @default undefined
   */
  readonly rovingFocus?: boolean;
  /** Accessible name when no visible label or `aria-labelledby` supplies one. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label the group. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe the group. @default undefined */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Compound ButtonGroup items. Receives grouping, availability, and navigation state. */
  default(props: ButtonGroupSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired after an enabled item is activated by pointer or keyboard. */
  press: [value: string, nativeEvent: MouseEvent];
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const items = new Set<ButtonGroupItemRegistration>();
const activeValue = shallowRef<string | null>(null);
const roleState = computed(() => role);
const disabledState = computed(() => disabled);
const orientationState = computed(() => orientation);
const rovingFocusState = computed(() => rovingFocus ?? roleState.value === "toolbar");
const state = computed<ButtonGroupState>(() => (disabledState.value ? "disabled" : "idle"));
const slotState = computed<ButtonGroupSlotState>(() => ({
  disabled: disabledState.value,
  orientation: orientationState.value,
  role: roleState.value,
  rovingFocus: rovingFocusState.value,
  state: state.value,
}));

watch([disabledState, orientationState, roleState, rovingFocusState], () => syncActiveValue(), {
  flush: "post",
  immediate: true,
});

function getElementNode(target: PrimitiveElement | null): Element | null {
  if (typeof Element !== "undefined" && target instanceof Element) return target;
  if (
    typeof Element !== "undefined" &&
    target != null &&
    "$el" in target &&
    target.$el instanceof Element
  ) {
    return target.$el;
  }
  return null;
}

function focusTarget(target: PrimitiveElement | null, options?: FocusOptions): boolean {
  const node = getElementNode(target);
  if (typeof HTMLElement !== "undefined" && node instanceof HTMLElement) {
    node.focus(options);
    return true;
  }
  if (target != null && "focus" in target && typeof target.focus === "function") {
    target.focus(options);
    return true;
  }
  return false;
}

function getOrderedItems(): readonly ButtonGroupItemRegistration[] {
  const enabled = [...items].filter((item) => !item.disabled.value);
  const root = getElementNode(element.value);
  if (root === null) return enabled;

  const positions = new Map<Element, number>();
  root
    .querySelectorAll('[data-vize-ui="button-group-item"]')
    .forEach((node, index) => positions.set(node, index));

  return enabled.sort((left, right) => {
    const leftOrder = positions.get(getElementNode(left.element.value) ?? root) ?? enabled.length;
    const rightOrder = positions.get(getElementNode(right.element.value) ?? root) ?? enabled.length;
    return leftOrder - rightOrder;
  });
}

function validateItemValue(item: ButtonGroupItemRegistration): void {
  const itemValue = item.value();
  for (const registered of items) {
    if (registered !== item && registered.value() === itemValue) {
      throw new Error("VIZE_UI_BUTTON_GROUP_VALUE_DUPLICATE");
    }
  }
}

function syncActiveValue(item?: ButtonGroupItemRegistration): void {
  if (item !== undefined) validateItemValue(item);
  if (disabledState.value || !rovingFocusState.value) {
    activeValue.value = null;
    return;
  }

  const enabled = getOrderedItems();
  if (enabled.some((item) => item.value() === activeValue.value)) return;
  activeValue.value = enabled[0]?.value() ?? null;
}

function focusItem(item: ButtonGroupItemRegistration, options?: FocusOptions): void {
  activeValue.value = item.value();
  focusTarget(item.element.value, options);
}

function focus(options?: FocusOptions): void {
  syncActiveValue();
  const enabled = getOrderedItems();
  const target = enabled.find((item) => item.value() === activeValue.value) ?? enabled[0];
  if (target !== undefined) focusItem(target, options);
}

function focusValue(value: string, options?: FocusOptions): boolean {
  const target = getOrderedItems().find((item) => item.value() === value);
  if (target === undefined) return false;
  focusItem(target, options);
  return true;
}

function moveFocus(
  fromValue: string,
  intent: ButtonGroupNavigationIntent,
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

function getItemState(itemDisabled: boolean): ButtonGroupItemState {
  return itemDisabled ? "disabled" : "idle";
}

function registerItem(item: ButtonGroupItemRegistration): () => void {
  validateItemValue(item);
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

function requestItemPress(value: string, nativeEvent: MouseEvent): void {
  if (!disabledState.value) emit("press", value, nativeEvent);
}

function setActiveValue(value: string): void {
  if (!disabledState.value) activeValue.value = value;
}

const context = buttonGroupContext.provide({
  activeValue,
  disabled: disabledState,
  getItemState,
  moveFocus,
  orientation: orientationState,
  registerItem,
  requestItemPress,
  role: roleState,
  rovingFocus: rovingFocusState,
  setActiveValue,
  state,
  syncActiveValue,
} satisfies ButtonGroupContextValue);

type ButtonGroupSetupExpose = Omit<
  ButtonGroupExpose,
  keyof ButtonGroupSlotState | "activeValue" | "element"
> & {
  readonly activeValue: Readonly<ShallowRef<string | null>>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ButtonGroupOrientation>;
  readonly role: ComputedRef<ButtonGroupRole>;
  readonly rovingFocus: ComputedRef<boolean>;
  readonly state: ComputedRef<ButtonGroupState>;
};

const exposed = {
  activeValue,
  disabled: disabledState,
  element,
  focus,
  focusValue,
  orientation: orientationState,
  role: roleState,
  rovingFocus: rovingFocusState,
  state,
} satisfies ButtonGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :role="context.role.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="context.role.value === 'toolbar' ? orientationState : undefined"
    :aria-disabled="disabledState ? 'true' : undefined"
    data-vize-ui="button-group"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-roving-focus="rovingFocusState ? 'true' : undefined"
    :data-role="context.role.value"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
