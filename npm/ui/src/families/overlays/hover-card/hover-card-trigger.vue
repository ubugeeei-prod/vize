<script setup lang="ts">
import { computed, onMounted, onScopeDispose, onUnmounted, useTemplateRef } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { useHover } from "../../interaction/hover/hover.ts";
import { createPointerGrace } from "../../interaction/pointer-grace/pointer-grace.ts";
import type { Point } from "../../interaction/pointer-grace/pointer-grace.ts";
import { hoverCardContext } from "./hover-card-context.ts";
import type { HoverCardSlotState, HoverCardTriggerExpose } from "./hover-card-types.ts";

/** Movement in CSS pixels that cancels a pending touch long-press. */
const touchSlop = 10;

const { as = "a", disabled = false } = defineProps<{
  /**
   * Element or component rendered as the trigger. Hover cards usually enhance
   * a link, so the default is a native anchor; pass `href` and other link
   * attributes as fallthrough attributes.
   *
   * @default "a"
   */
  readonly as?: PrimitiveAs;

  /**
   * Ignore pointer, focus, and touch intent from this trigger.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Trigger contents. Receives the current HoverCard state. */
  default(props: HoverCardSlotState): unknown;
}>();

const context = hoverCardContext.use();
const element = useTemplateRef<HTMLElement>("element");
const disabledState = computed(() => disabled || context.disabled.value);
const grace = createPointerGrace({ delay: 0 });
const hover = useHover({
  isDisabled: disabledState,
  onHoverStart: (event) => {
    stopGraceTracking();
    context.scheduleOpen(event.originalEvent, "hover");
  },
  onHoverEnd: (event) => {
    context.scheduleClose(event.originalEvent);
    startGraceTracking(event.x, event.y);
  },
});
let graceDocument: Document | null = null;
let touchTimer: ReturnType<typeof setTimeout> | null = null;
let touchOrigin: Point | null = null;
let suppressClick = false;

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

onScopeDispose(() => {
  stopGraceTracking();
  cancelTouch();
  grace.dispose();
});

function readRect(target: Element) {
  const rect = target.getBoundingClientRect();
  return { height: rect.height, width: rect.width, x: rect.left, y: rect.top };
}

/**
 * Keep the card open while the pointer travels from the trigger toward the
 * card through the safe triangle, and close it once the pointer leaves.
 */
function onGracePointerMove(event: PointerEvent): void {
  const content = context.contentElement.value;
  if (!context.open.value || !content) {
    stopGraceTracking();
    return;
  }
  const point = { x: event.clientX, y: event.clientY };
  grace.setTarget(readRect(content));
  if (grace.contains(point)) context.cancelPending();
  else context.scheduleClose(event);
}

function startGraceTracking(x: number | null, y: number | null): void {
  const content = context.contentElement.value;
  if (x === null || y === null || !context.open.value || !content) return;
  stopGraceTracking();
  grace.setOrigin({ x, y });
  grace.setTarget(readRect(content));
  graceDocument = content.ownerDocument;
  graceDocument.addEventListener("pointermove", onGracePointerMove);
}

function stopGraceTracking(): void {
  graceDocument?.removeEventListener("pointermove", onGracePointerMove);
  graceDocument = null;
  grace.setOrigin(null);
  grace.setTarget(null);
}

function cancelTouch(): void {
  if (touchTimer !== null) clearTimeout(touchTimer);
  touchTimer = null;
  touchOrigin = null;
}

function onPointerdown(event: PointerEvent): void {
  if (event.pointerType !== "touch" || disabledState.value) return;
  if (context.touchBehavior.value !== "long-press") return;
  cancelTouch();
  suppressClick = false;
  touchOrigin = { x: event.clientX, y: event.clientY };
  touchTimer = setTimeout(() => {
    touchTimer = null;
    suppressClick = true;
    context.setOpen(true, event, "long-press");
  }, context.longPressDelay.value);
}

function onPointermove(event: PointerEvent): void {
  if (touchOrigin === null || event.pointerType !== "touch") return;
  const distance = Math.hypot(event.clientX - touchOrigin.x, event.clientY - touchOrigin.y);
  if (distance > touchSlop) cancelTouch();
}

function onPointerEnd(): void {
  cancelTouch();
}

function onClick(event: MouseEvent): void {
  if (!suppressClick) return;
  suppressClick = false;
  event.preventDefault();
}

function onContextmenu(event: MouseEvent): void {
  if (touchTimer !== null || suppressClick) event.preventDefault();
}

function onFocus(event: FocusEvent): void {
  if (!disabledState.value) context.scheduleOpen(event, "focus");
}

function onBlur(event: FocusEvent): void {
  const next = event.relatedTarget;
  const content = context.contentElement.value;
  if (content && next instanceof Node && content.contains(next)) return;
  context.scheduleClose(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Escape" || event.defaultPrevented || event.isComposing) return;
  if (context.open.value && context.setOpen(false, event)) event.preventDefault();
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type HoverCardTriggerSetupExpose = Omit<HoverCardTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies HoverCardTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="context.triggerId.value"
    ref="element"
    v-bind="hover.hoverProps"
    :aria-describedby="context.open.value ? context.contentId.value : undefined"
    data-vize-ui="hover-card-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @focus="onFocus"
    @blur="onBlur"
    @keydown="onKeydown"
    @pointerdown="onPointerdown"
    @pointermove="onPointermove"
    @pointerup="onPointerEnd"
    @pointercancel="onPointerEnd"
    @click="onClick"
    @contextmenu="onContextmenu"
  >
    <slot
      :disabled="disabledState"
      :open="context.open.value"
      :reason="context.reason.value"
      :state="context.state.value"
    />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
