<script setup lang="ts">
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type { PrimitiveElement } from "../../../primitive.ts";
import type { ToolbarExpose } from "./toolbar-contracts.ts";
import { toolbarContext } from "./toolbar-context.ts";
import type {
  ToolbarContextValue,
  ToolbarItemRegistration,
  ToolbarNavigationIntent,
} from "./toolbar-context.ts";
import type {
  ToolbarDirection,
  ToolbarEmits,
  ToolbarItemState,
  ToolbarOrientation,
  ToolbarProps,
  ToolbarSlotState,
  ToolbarState,
  ToolbarStyle,
} from "./toolbar-types.ts";

const {
  as = "div",
  disabled = false,
  orientation = "horizontal",
  dir = "ltr",
  loop = true,
  rovingFocus = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<ToolbarProps>();

defineSlots<{
  /** Renders grouped action controls with toolbar navigation state. */
  default(props: ToolbarSlotState): unknown;
}>();

const emit = defineEmits<ToolbarEmits>();

const element = useTemplateRef<PrimitiveElement>("element");
const items = new Set<ToolbarItemRegistration>();
const activeValue = shallowRef<string | null>(null);
const disabledState = computed(() => disabled);
const orientationState = computed<ToolbarOrientation>(() => orientation);
const dirState = computed<ToolbarDirection>(() => dir);
const rovingFocusState = computed(() => rovingFocus);
const state = computed<ToolbarState>(() => (disabledState.value ? "disabled" : "idle"));
const toolbarStyle = computed<ToolbarStyle>(() => ({
  "--vize-ui-toolbar-orientation": orientationState.value,
}));
const intrinsicProps = computed(() => ({ style: toolbarStyle.value }));
const slotState = computed<ToolbarSlotState>(() => ({
  dir: dirState.value,
  disabled: disabledState.value,
  orientation: orientationState.value,
  rovingFocus: rovingFocusState.value,
  state: state.value,
  style: toolbarStyle.value,
}));

watch([disabledState, orientationState, dirState, rovingFocusState], () => syncActiveValue(), {
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

function getOrderedItems(): readonly ToolbarItemRegistration[] {
  const enabled = [...items].filter((item) => !item.disabled.value);
  const root = getElementNode(element.value);
  if (root === null) return enabled;

  const positions = new Map<Element, number>();
  root
    .querySelectorAll('[data-vize-ui="toolbar-item"]')
    .forEach((node, index) => positions.set(node, index));

  return enabled.sort((left, right) => {
    const leftOrder = positions.get(getElementNode(left.element.value) ?? root) ?? enabled.length;
    const rightOrder = positions.get(getElementNode(right.element.value) ?? root) ?? enabled.length;
    return leftOrder - rightOrder;
  });
}

function validateItemValue(item: ToolbarItemRegistration): void {
  const itemValue = item.value();
  for (const registered of items) {
    if (registered !== item && registered.value() === itemValue) {
      throw new Error("VIZE_UI_TOOLBAR_VALUE_DUPLICATE");
    }
  }
}

function syncActiveValue(item?: ToolbarItemRegistration): void {
  if (item !== undefined) validateItemValue(item);
  if (disabledState.value || !rovingFocusState.value) {
    activeValue.value = null;
    return;
  }

  const enabled = getOrderedItems();
  if (enabled.some((candidate) => candidate.value() === activeValue.value)) return;
  activeValue.value = enabled[0]?.value() ?? null;
}

function focusItem(item: ToolbarItemRegistration, options?: FocusOptions): void {
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
  intent: ToolbarNavigationIntent,
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

function getItemState(itemDisabled: boolean): ToolbarItemState {
  return itemDisabled ? "disabled" : "idle";
}

function registerItem(item: ToolbarItemRegistration): () => void {
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

toolbarContext.provide({
  activeValue,
  dir: dirState,
  disabled: disabledState,
  getItemState,
  moveFocus,
  orientation: orientationState,
  registerItem,
  requestItemPress,
  rovingFocus: rovingFocusState,
  setActiveValue,
  state,
  syncActiveValue,
} satisfies ToolbarContextValue);

type ToolbarSetupExpose = Omit<
  ToolbarExpose,
  keyof ToolbarSlotState | "activeValue" | "element"
> & {
  readonly activeValue: Readonly<ShallowRef<string | null>>;
  readonly dir: ComputedRef<ToolbarDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ToolbarOrientation>;
  readonly rovingFocus: ComputedRef<boolean>;
  readonly state: ComputedRef<ToolbarState>;
  readonly style: ComputedRef<ToolbarStyle>;
};

const exposed = {
  activeValue,
  dir: dirState,
  disabled: disabledState,
  element,
  focus,
  focusValue,
  orientation: orientationState,
  rovingFocus: rovingFocusState,
  state,
  style: toolbarStyle,
} satisfies ToolbarSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    role="toolbar"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="orientationState"
    :aria-disabled="disabledState ? 'true' : undefined"
    :dir="dirState"
    data-vize-ui="toolbar"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-orientation="orientationState"
    :data-roving-focus="rovingFocusState ? 'true' : undefined"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Layout, spacing, grouping dividers, and affordances remain consumer-owned. */
</style>
