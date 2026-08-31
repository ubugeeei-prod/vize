<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createDismissableLayer } from "../dismissable-layer/dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
} from "../dismissable-layer/dismissable-layer.ts";
import Portal from "../portal/portal.vue";
import Positioner from "../positioner/positioner.vue";
import type { Placement, PositionerStrategy, Rect } from "../positioner/positioner.ts";
import Presence from "../presence/presence.vue";
import { tooltipContext } from "./tooltip-context.ts";
import type {
  TooltipContentExpose,
  TooltipContentSlotState,
  TooltipState,
} from "./tooltip-types.ts";

interface TooltipPositionerProps {
  readonly reference: HTMLButtonElement | null;
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
  placement = "top",
  strategy = "fixed",
  offset = 6,
  collisionPadding = 4,
  flip = true,
  shift = true,
  hide = true,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
  closeOnEscape = true,
  ariaLabel = undefined,
} = defineProps<{
  /** Keep the content mounted while the tooltip is closed. @default false */
  readonly forceMount?: boolean;
  /** CSS selector or element the tooltip layer is moved into. @default "body" */
  readonly to?: string | HTMLElement;
  /** Render in place instead of teleporting. @default false */
  readonly portalDisabled?: boolean;
  /** Keep content in place until the target exists, avoiding SSR mismatch. @default true */
  readonly defer?: boolean;
  /** Preferred placement before collision handling. @default "top" */
  readonly placement?: Placement;
  /** CSS positioning mode published on the floating host. @default "fixed" */
  readonly strategy?: PositionerStrategy;
  /** Gap on the main axis between trigger and content. @default 6 */
  readonly offset?: number;
  /** Viewport padding the floating element should not cross. @default 4 */
  readonly collisionPadding?: number;
  /** Flip to the opposite side when the preferred side overflows more. @default true */
  readonly flip?: boolean;
  /** Shift the floating box back into the viewport after flip. @default true */
  readonly shift?: boolean;
  /** Hide when the trigger no longer intersects the viewport. @default true */
  readonly hide?: boolean;
  /** Recalculate while ancestors scroll. @default true */
  readonly updateOnScroll?: boolean;
  /** Recalculate when the document or visual viewport resizes. @default true */
  readonly updateOnResize?: boolean;
  /** Viewport used for flip, shift, and hide. @default undefined */
  readonly viewport?: Rect;
  /** Let Escape request dismissal while the tooltip is open. @default true */
  readonly closeOnEscape?: boolean;
  /** Accessible name when visible text is not enough. @default undefined */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** Tooltip content. Receives open state and resolved placement. */
  default(props: TooltipContentSlotState): unknown;
}>();

const context = tooltipContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const contentSlotState = computed<TooltipContentSlotState>(() => ({
  disabled: context.disabled.value,
  open: context.open.value,
  placement,
  state: context.state.value,
}));
const positionerProps = computed<TooltipPositionerProps>(() => {
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
  branches: () =>
    [context.triggerElement.value].filter((value): value is HTMLButtonElement => !!value),
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape,
  outsideFocus: false,
  outsidePointerDown: false,
  onEscapeKeyDown: (event) => emit("escape-key-down", event),
  onDismiss: (event) => {
    emit("dismiss", event);
    context.close(event.originalEvent);
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

type TooltipContentSetupExpose = Omit<
  TooltipContentExpose,
  "disabled" | "element" | "open" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<TooltipState>;
};

const exposed = {
  disabled: context.disabled,
  element,
  open: context.open,
  state: context.state,
} satisfies TooltipContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="tooltip-content-host"
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
            role="tooltip"
            :aria-label="ariaLabel"
            data-vize-ui="tooltip-content"
            part="content"
            :hidden="context.open.value ? undefined : true"
            :data-state="context.state.value"
            :data-placement="placement"
            :data-top-layer="dismissableLayer.isTopLayer.value ? 'true' : 'false'"
          >
            <slot v-bind="contentSlotState" />
          </div>
        </Positioner>
      </Presence>
    </Portal>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
