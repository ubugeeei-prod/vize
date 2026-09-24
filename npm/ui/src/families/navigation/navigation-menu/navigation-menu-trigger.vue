<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";

import { navigationMenuContext, navigationMenuItemContext } from "./navigation-menu-context.ts";
import type { NavigationMenuItemSlotState } from "./navigation-menu-types.ts";

const { disabled = false } = defineProps<{
  /**
   * Disable the trigger and its flyout.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Trigger label. Receives the open state of the owning item. */
  default(props: NavigationMenuItemSlotState): unknown;
}>();

const context = navigationMenuContext.use();
const item = navigationMenuItemContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const triggerId = computed(() => context.getTriggerId(item.value.value));
const contentId = computed(() => context.getContentId(item.value.value));
const slotState = computed<NavigationMenuItemSlotState>(() => ({
  open: item.open.value,
  state: item.open.value ? "open" : "closed",
  value: item.value.value,
}));

watch(
  item.value,
  (value, _previous, onCleanup) => onCleanup(context.registerTrigger(value, element)),
  {
    flush: "sync",
    immediate: true,
  },
);

let openedByPointer = false;
watch(item.open, (open) => {
  if (!open) openedByPointer = false;
});

function isMouse(event: PointerEvent): boolean {
  return event.pointerType === "mouse" || event.pointerType === "pen";
}

function onPointerenter(event: PointerEvent): void {
  if (disabled || !isMouse(event)) return;
  const wasOpen = item.open.value;
  context.onTriggerEnter(item.value.value);
  if (!wasOpen) openedByPointer = true;
}

function onPointerleave(event: PointerEvent): void {
  if (disabled || !isMouse(event)) return;
  context.onPointerLeave();
}

function onClick(): void {
  if (disabled) return;
  // A hover-opened flyout stays open on the click that usually follows the hover.
  if (item.open.value && openedByPointer) {
    openedByPointer = false;
    return;
  }
  context.toggle(item.value.value);
}

function onKeydown(event: KeyboardEvent): void {
  if (disabled) return;
  const openKey =
    context.orientation.value === "horizontal"
      ? "ArrowDown"
      : context.dir.value === "rtl"
        ? "ArrowLeft"
        : "ArrowRight";
  if (event.key !== openKey) return;
  event.preventDefault();
  context.setOpen(item.value.value, "keyboard");
  void nextTick(() => context.focusContent(item.value.value));
}

const triggerProps = computed<{
  readonly onClick: () => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onPointerenter: (event: PointerEvent) => void;
  readonly onPointerleave: (event: PointerEvent) => void;
}>(() => ({ onClick, onKeydown, onPointerenter, onPointerleave }));
</script>

<template>
  <button
    v-bind="triggerProps"
    :id="triggerId"
    ref="element"
    type="button"
    :disabled="disabled"
    :aria-expanded="item.open.value ? 'true' : 'false'"
    :aria-controls="contentId"
    data-navigation-menu-entry=""
    data-vize-ui="navigation-menu-trigger"
    part="trigger"
    :data-state="item.open.value ? 'open' : 'closed'"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
