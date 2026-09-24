<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { dialogContext } from "../dialog/dialog.ts";
import { drawerContext } from "./drawer-context.ts";
import type { DrawerHandleExpose, DrawerSide, DrawerSlotState } from "./drawer-types.ts";

const { disabled = false, ariaLabel = "Resize drawer" } = defineProps<{
  /**
   * Remove the handle from activation and sequential keyboard focus. Dragging from content still works.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name of the handle button.
   *
   * @default "Resize drawer"
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before the handle steps to the next snap point. Call `preventDefault()` to keep it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Optional grip contents. Receives the current Drawer state. */
  default(props: DrawerSlotState): unknown;
}>();

const context = dialogContext.use();
const drawer = drawerContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const moreVisibleKey: Readonly<Record<DrawerSide, string>> = {
  bottom: "ArrowUp",
  left: "ArrowRight",
  right: "ArrowLeft",
  top: "ArrowDown",
};
const lessVisibleKey: Readonly<Record<DrawerSide, string>> = {
  bottom: "ArrowDown",
  left: "ArrowLeft",
  right: "ArrowRight",
  top: "ArrowUp",
};
const slotState = computed<DrawerSlotState>(() => ({
  activeSnapPoint: drawer.activeSnapPoint.value,
  dragging: drawer.dragging.value,
  modal: context.modal.value,
  open: context.open.value,
  side: drawer.side.value,
  state: context.state.value,
}));

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) drawer.stepSnapPoint(1, true, event);
}

function onKeydown(event: KeyboardEvent): void {
  if (disabled || event.defaultPrevented || event.isComposing) return;
  const side = drawer.side.value;
  let direction: 1 | -1 | null = null;
  if (event.key === moreVisibleKey[side]) direction = 1;
  else if (event.key === lessVisibleKey[side]) direction = -1;
  if (direction === null || drawer.snapPoints.value.length === 0) return;
  event.preventDefault();
  drawer.stepSnapPoint(direction, false, event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type DrawerHandleSetupExpose = Omit<DrawerHandleExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies DrawerHandleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    :aria-controls="context.contentId.value"
    data-vize-ui="drawer-handle"
    part="handle"
    :data-state="context.state.value"
    :data-side="drawer.side.value"
    :data-snap-point="
      drawer.activeSnapPoint.value === null ? undefined : String(drawer.activeSnapPoint.value)
    "
    :data-dragging="drawer.dragging.value ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
