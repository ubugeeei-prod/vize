<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import { useDismissableLayer } from "../../overlays/dismissable-layer/dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../../overlays/dismissable-layer/dismissable-layer.ts";
import Portal from "../../overlays/portal/portal.vue";
import Positioner from "../../overlays/positioner/positioner.vue";
import type { Placement, PositionerStrategy } from "../../overlays/positioner/positioner.ts";
import Presence from "../../overlays/presence/presence.vue";
import { cascaderContext } from "./cascader-context.ts";
import type { CascaderContentSlotState } from "./cascader-types.ts";

const {
  placement = "bottom-start",
  strategy = "fixed",
  offset = 4,
  collisionPadding = 8,
  flip = true,
  shift = true,
  size = true,
  to = "body",
  portalDisabled = false,
  defer = true,
  forceMount = false,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Preferred placement relative to the trigger before collision handling.
   *
   * @default "bottom-start"
   */
  readonly placement?: Placement;

  /**
   * CSS positioning mode published on the floating host.
   *
   * @default "fixed"
   */
  readonly strategy?: PositionerStrategy;

  /**
   * Gap between the trigger and the popup.
   *
   * @default 4
   */
  readonly offset?: number;

  /**
   * Viewport padding the popup should not cross.
   *
   * @default 8
   */
  readonly collisionPadding?: number;

  /**
   * Flip to the opposite side when the preferred side overflows more.
   *
   * @default true
   */
  readonly flip?: boolean;

  /**
   * Shift the popup back into the viewport.
   *
   * @default true
   */
  readonly shift?: boolean;

  /**
   * Publish available-size CSS variables and constrain the host.
   *
   * @default true
   */
  readonly size?: boolean;

  /**
   * CSS selector or element the popup is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render in place instead of teleporting.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;

  /**
   * Keep content in place until the portal target exists, avoiding SSR mismatch.
   *
   * @default true
   */
  readonly defer?: boolean;

  /**
   * Keep items mounted (hidden) while closed.
   *
   * @default false
   */
  readonly forceMount?: boolean;

  /**
   * Accessible name of the popup region holding the columns.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before Escape closes the popup. Call `preventDefault()` to keep it open. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];

  /** Fired before an outside pointer-down closes the popup. Call `preventDefault()` to keep it open. */
  "pointer-down-outside": [event: DismissableLayerPointerDownOutsideEvent];

  /** Fired after an unprevented dismissal request closes the popup. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** One `CascaderColumn` per entry of the root `columns`. Receives open state and placement. */
  default(props: CascaderContentSlotState): unknown;
}>();

const context = cascaderContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const positionerProps = computed(() => ({
  collisionPadding,
  flip,
  offset,
  placement,
  reference: context.triggerElement.value,
  shift,
  size,
  strategy,
}));
const surfaceProps = computed(() => ({
  onMousedown: keepTriggerFocus,
  onPointerdown: keepTriggerFocus,
}));

const layer = useDismissableLayer({
  root: element,
  branches: () => (context.triggerElement.value === null ? [] : [context.triggerElement.value]),
  enabled: () => context.open.value,
  onEscapeKeyDown: (event) => emit("escape-key-down", event),
  onPointerDownOutside: (event) => emit("pointer-down-outside", event),
  onDismiss: (event) => {
    emit("dismiss", event);
    context.setOpen(false, event.originalEvent);
  },
});

watch(
  element,
  (next, previous) => {
    if (previous !== null && context.contentElement.value === previous) {
      context.contentElement.value = null;
    }
    if (next !== null) context.contentElement.value = next;
    layer.refresh();
  },
  { flush: "post" },
);

onUnmounted(() => {
  if (context.contentElement.value === element.value) context.contentElement.value = null;
});

// Options never take DOM focus: the trigger keeps it and exposes the
// highlighted option through aria-activedescendant.
function keepTriggerFocus(event: MouseEvent | PointerEvent): void {
  if (event.button === 0) event.preventDefault();
}

function isPlacement(value: string): value is Placement {
  return /^(?:top|right|bottom|left)(?:-(?:start|center|end))?$/u.test(value);
}

function placementOf(value: unknown): Placement {
  return typeof value === "string" && isPlacement(value) ? value : placement;
}

function slotState(resolved: unknown): CascaderContentSlotState {
  return { open: context.open.value, placement: placementOf(resolved) };
}

defineExpose({ element });
</script>

<template>
  <div
    data-vize-ui="cascader-content-host"
    part="content-host"
    :hidden="context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="context.open.value" :force-mount>
        <Positioner v-bind="positionerProps">
          <template #default="{ placement: resolved }: { placement: Placement }">
            <div
              :id="context.contentId.value"
              ref="element"
              v-bind="{ ...layer.layerProps, ...surfaceProps }"
              :aria-label="ariaLabel"
              data-vize-ui="cascader-content"
              part="content"
              :hidden="context.open.value ? undefined : true"
              :data-state="context.state.value"
              :data-placement="placementOf(resolved)"
            >
              <slot v-bind="slotState(resolved)" />
            </div>
          </template>
        </Positioner>
      </Presence>
    </Portal>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
