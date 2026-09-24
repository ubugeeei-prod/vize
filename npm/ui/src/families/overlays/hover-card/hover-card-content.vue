<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { createDismissableLayer } from "../dismissable-layer/dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../dismissable-layer/dismissable-layer.ts";
import Portal from "../portal/portal.vue";
import Positioner from "../positioner/positioner.vue";
import type { Placement, PositionerStrategy, Rect } from "../positioner/positioner.ts";
import Presence from "../presence/presence.vue";
import { hoverCardContext } from "./hover-card-context.ts";
import type {
  HoverCardContentExpose,
  HoverCardContentSlotState,
  HoverCardOpenReason,
  HoverCardState,
} from "./hover-card-types.ts";

interface HoverCardPositionerProps {
  readonly reference: HTMLElement | null;
  readonly placement: Placement;
  readonly strategy: PositionerStrategy;
  readonly offset: number;
  readonly collisionPadding: number;
  readonly flip: boolean;
  readonly shift: boolean;
  readonly hide: boolean;
  readonly updateOnScroll: boolean;
  readonly updateOnResize: boolean;
  readonly viewport?: Rect;
}

const {
  forceMount = false,
  to = "body",
  portalDisabled = false,
  defer = true,
  placement = "bottom",
  strategy = "fixed",
  offset = 8,
  collisionPadding = 8,
  flip = true,
  shift = true,
  hide = true,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  ariaLabel = undefined,
} = defineProps<{
  /** Keep the card mounted while closed. @default false */
  readonly forceMount?: boolean;
  /** CSS selector or element the card layer is moved into. @default "body" */
  readonly to?: string | HTMLElement;
  /** Render in place instead of teleporting. @default false */
  readonly portalDisabled?: boolean;
  /** Keep content in place until the target exists, avoiding SSR mismatch. @default true */
  readonly defer?: boolean;
  /** Preferred placement before collision handling. @default "bottom" */
  readonly placement?: Placement;
  /** CSS positioning mode published on the floating host. @default "fixed" */
  readonly strategy?: PositionerStrategy;
  /** Gap on the main axis between trigger and card. @default 8 */
  readonly offset?: number;
  /** Viewport padding the card should not cross. @default 8 */
  readonly collisionPadding?: number;
  /** Flip to the opposite side when the preferred side overflows more. @default true */
  readonly flip?: boolean;
  /** Shift the card back into the viewport after flip. @default true */
  readonly shift?: boolean;
  /** Hide when the trigger no longer intersects the viewport. @default true */
  readonly hide?: boolean;
  /** Recalculate while ancestors scroll. @default true */
  readonly updateOnScroll?: boolean;
  /** Recalculate when the document or visual viewport resizes. @default true */
  readonly updateOnResize?: boolean;
  /** Viewport used for flip, shift, and hide. @default undefined */
  readonly viewport?: Rect;
  /** Let Escape close the card. @default true */
  readonly closeOnEscape?: boolean;
  /** Let an outside pointer-down close the card. @default true */
  readonly closeOnPointerDownOutside?: boolean;
  /** Accessible name for the card when its contents do not provide one. @default undefined */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];
  /** Fired before an outside pointer-down requests dismissal. */
  "pointer-down-outside": [event: DismissableLayerPointerDownOutsideEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** Card contents. Receives open state, open reason, and preferred placement. */
  default(props: HoverCardContentSlotState): unknown;
}>();

const context = hoverCardContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const slotState = computed<HoverCardContentSlotState>(() => ({
  disabled: context.disabled.value,
  open: context.open.value,
  placement,
  reason: context.reason.value,
  state: context.state.value,
}));
const positionerProps = computed<HoverCardPositionerProps>(() => {
  const base = {
    collisionPadding,
    flip,
    hide,
    offset,
    placement,
    reference: context.triggerElement.value,
    shift,
    strategy,
    updateOnResize,
    updateOnScroll,
  };
  return viewport === undefined ? base : { ...base, viewport };
});
const dismissableLayer = createDismissableLayer({
  root: element,
  branches: () => [context.triggerElement.value].filter((value): value is HTMLElement => !!value),
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape,
  outsideFocus: false,
  outsidePointerDown: () => closeOnPointerDownOutside,
  onEscapeKeyDown: (event) => emit("escape-key-down", event),
  onPointerDownOutside: (event) => emit("pointer-down-outside", event),
  onDismiss: (event) => {
    emit("dismiss", event);
    context.setOpen(false, event.originalEvent);
  },
});
let mounted = false;

function syncDismissableLayer(): void {
  if (!mounted || !context.open.value || !element.value) {
    dismissableLayer.deactivate();
    return;
  }
  dismissableLayer.activate();
}

function onPointerenter(event: PointerEvent): void {
  if (event.pointerType !== "touch") context.cancelPending();
}

function onPointerleave(event: PointerEvent): void {
  if (event.pointerType === "touch") return;
  const active = element.value?.ownerDocument.activeElement ?? null;
  if (active && element.value?.contains(active)) return;
  context.scheduleClose(event);
}

function onFocusin(): void {
  context.cancelPending();
}

function onFocusout(event: FocusEvent): void {
  const next = event.relatedTarget;
  if (next instanceof Node) {
    if (element.value?.contains(next)) return;
    if (context.triggerElement.value?.contains(next)) return;
  }
  context.scheduleClose(event);
}

watch(
  element,
  (next, previous) => {
    if (previous && context.contentElement.value === previous) context.contentElement.value = null;
    if (next) context.contentElement.value = next;
    syncDismissableLayer();
  },
  { flush: "post" },
);
watch(() => context.open.value, syncDismissableLayer, { flush: "post" });

onMounted(() => {
  mounted = true;
  syncDismissableLayer();
});

onUnmounted(() => {
  mounted = false;
  dismissableLayer.dispose();
  if (context.contentElement.value === element.value) context.contentElement.value = null;
});

type HoverCardContentSetupExpose = Omit<
  HoverCardContentExpose,
  "disabled" | "element" | "open" | "reason" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly open: ComputedRef<boolean>;
  readonly reason: Readonly<ShallowRef<HoverCardOpenReason | null>>;
  readonly state: ComputedRef<HoverCardState>;
};

const exposed = {
  disabled: context.disabled,
  element,
  open: context.open,
  reason: context.reason,
  state: context.state,
} satisfies HoverCardContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    data-vize-ui="hover-card-content-host"
    part="content-host"
    :hidden="present && context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="context.open.value" :force-mount>
        <Positioner v-bind="positionerProps">
          <div
            :id="context.contentId.value"
            ref="element"
            v-bind="dismissableLayer.layerProps"
            :aria-label="ariaLabel"
            data-vize-ui="hover-card-content"
            part="content"
            :hidden="context.open.value ? undefined : true"
            :data-state="context.state.value"
            :data-placement="placement"
            :data-reason="context.reason.value ?? undefined"
            :data-top-layer="dismissableLayer.isTopLayer.value ? 'true' : 'false'"
            @pointerenter="onPointerenter"
            @pointerleave="onPointerleave"
            @focusin="onFocusin"
            @focusout="onFocusout"
          >
            <slot v-bind="slotState" />
          </div>
        </Positioner>
      </Presence>
    </Portal>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
