<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useLongPress } from "../../interaction/long-press/long-press.ts";
import { menuLevelContext, menuTreeContext } from "../menu/menu-context.ts";
import { contextMenuContext } from "./context-menu-context.ts";
import type {
  ContextMenuTriggerExpose,
  ContextMenuTriggerSlotState,
} from "./context-menu-types.ts";

const { disabled = false, longPressDelay = 700 } = defineProps<{
  /**
   * Let the native browser context menu show for this region.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Milliseconds a touch must stay down before the menu opens at the touch point.
   *
   * @default 700
   */
  readonly longPressDelay?: number;
}>();

const emit = defineEmits<{
  /** Fired before a `contextmenu` request opens the menu; call `preventDefault()` to skip it. */
  contextmenu: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Region contents. Receives the menu open state and trigger availability. */
  default(props: ContextMenuTriggerSlotState): unknown;
}>();

const tree = menuTreeContext.use();
const level = menuLevelContext.use();
const context = contextMenuContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const disabledState = computed(() => disabled || context.disabled.value);

const longPress = useLongPress({
  pointerType: "touch",
  threshold: () => longPressDelay,
  isDisabled: () => disabledState.value,
  onLongPress: (event) =>
    context.openAt({ x: event.x ?? 0, y: event.y ?? 0 }, event.originalEvent, "content"),
});
const pressProps = longPress.longPressProps;

function onContextmenu(event: MouseEvent): void {
  if (disabledState.value) return;
  emit("contextmenu", event);
  if (event.defaultPrevented) return;
  event.preventDefault();
  context.openAt({ x: event.clientX, y: event.clientY }, event, "content");
}

function onKeydown(event: KeyboardEvent): void {
  if (disabledState.value || event.defaultPrevented) return;
  const request = event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey);
  if (!request) return;
  event.preventDefault();
  const View = element.value?.ownerDocument.defaultView;
  const source = View && event.target instanceof View.Element ? event.target : element.value;
  const rect = source?.getBoundingClientRect();
  context.openAt({ x: rect?.left ?? 0, y: rect?.bottom ?? 0 }, event, "first");
}

// Only the pointer and touch recognizers are bound: keyboard requests and the
// native `contextmenu` event are handled here with anchor-point semantics.
const triggerProps = Object.freeze({
  onContextmenu,
  onKeydown,
  onPointercancel: pressProps.onPointercancel,
  onPointerdown: pressProps.onPointerdown,
  onPointermove: pressProps.onPointermove,
  onPointerup: pressProps.onPointerup,
  onTouchcancel: pressProps.onTouchcancel,
  onTouchend: pressProps.onTouchend,
  onTouchmove: pressProps.onTouchmove,
  onTouchstart: pressProps.onTouchstart,
});

defineExpose({ element } satisfies Record<keyof ContextMenuTriggerExpose, unknown>);
</script>

<template>
  <span
    v-bind="triggerProps"
    ref="element"
    data-vize-ui="context-menu-trigger"
    part="trigger"
    :data-state="level.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot
      :dir="tree.dir.value"
      :disabled="disabledState"
      :modal="tree.modal.value"
      :open="level.open.value"
      :state="level.state.value"
    />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
