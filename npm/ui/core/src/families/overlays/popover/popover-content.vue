<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { createDismissableLayer } from "../../../dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../../../dismissable-layer.ts";
import { createFocusGuards } from "../../../focus-guards.ts";
import { createFocusScope } from "../../../focus-scope.ts";
import { createInertOutside } from "../../../inert-outside.ts";
import Portal from "../../../portal.vue";
import Positioner from "../../../positioner.vue";
import type { Placement, PositionerStrategy, Rect } from "../../../positioner.ts";
import Presence from "../../../presence.vue";
import { popoverContext } from "./popover-context.ts";
import {
  createPopoverSlotState,
  existingPopoverElements,
  popoverAlignFromPlacement,
  popoverSideFromPlacement,
  resolvePopoverSlotPlacement,
  usePopoverContentLifecycle,
  withViewport,
} from "./popover-content-runtime.ts";
import type {
  PopoverAutoFocusEvent,
  PopoverContentExpose,
  PopoverContentSlotState,
  PopoverState,
} from "./popover-types.ts";
import { createScrollLock } from "../../../scroll-lock.ts";

const {
  forceMount = false,
  to = "body",
  portalDisabled = false,
  defer = true,
  placement = "bottom",
  strategy = "fixed",
  offset = 8,
  collisionPadding = 4,
  arrowPadding = 0,
  direction = "ltr",
  flip = true,
  shift = true,
  size = false,
  safeArea = false,
  hide = true,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
  trapFocus = true,
  autoFocus = true,
  restoreFocus = true,
  inertOutside = true,
  lockScroll = true,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  closeOnFocusOutside = true,
  initialFocus = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Keep the content mounted while the popover is closed. @default false */
  readonly forceMount?: boolean;
  /** CSS selector or element the popover layer is moved into. @default "body" */
  readonly to?: string | HTMLElement;
  /** Render in place instead of teleporting. @default false */
  readonly portalDisabled?: boolean;
  /** Keep content in place until the target exists, avoiding SSR mismatch. @default true */
  readonly defer?: boolean;
  /** Preferred placement before collision handling. @default "bottom" */
  readonly placement?: Placement;
  /** CSS positioning mode published on the floating host. @default "fixed" */
  readonly strategy?: PositionerStrategy;
  /** Gap on the main axis between trigger and content. @default 8 */
  readonly offset?: number;
  /** Viewport padding the floating element should not cross. @default 4 */
  readonly collisionPadding?: number;
  /** Inset kept between the arrow and floating edges. @default 0 */
  readonly arrowPadding?: number;
  /** Writing direction used to resolve start/end alignment. @default "ltr" */
  readonly direction?: "ltr" | "rtl";
  /** Flip to the opposite side when the preferred side overflows more. @default true */
  readonly flip?: boolean;
  /** Shift the floating box back into the viewport after flip. @default true */
  readonly shift?: boolean;
  /** Constrain the host and publish positioner available-size CSS variables. @default false */
  readonly size?: boolean;
  /** Keep floating content clear of safe-area insets. @default false */
  readonly safeArea?: boolean;
  /** Hide when the trigger no longer intersects the viewport. @default true */
  readonly hide?: boolean;
  /** Recalculate while ancestors scroll. @default true */
  readonly updateOnScroll?: boolean;
  /** Recalculate when the document or visual viewport resizes. @default true */
  readonly updateOnResize?: boolean;
  /** Viewport used for flip, shift, and hide. @default undefined */
  readonly viewport?: Rect;
  /** Contain focus inside an open modal popover. @default true */
  readonly trapFocus?: boolean;
  /** Move focus into content when it opens. @default true */
  readonly autoFocus?: boolean;
  /** Restore focus when content closes. @default true */
  readonly restoreFocus?: boolean;
  /** Make outside content inert while the modal popover is open. @default true */
  readonly inertOutside?: boolean;
  /** Lock document scroll while the modal popover is open. @default true */
  readonly lockScroll?: boolean;
  /** Let Escape request dismissal while the popover is open. @default true */
  readonly closeOnEscape?: boolean;
  /** Let outside pointer-down request dismissal. @default true */
  readonly closeOnPointerDownOutside?: boolean;
  /** Let outside focus movement request dismissal. @default true */
  readonly closeOnFocusOutside?: boolean;
  /** Preferred initial focus target inside the content. @default undefined */
  readonly initialFocus?: () => HTMLElement | null | undefined;
  /** Accessible name when visible text is not enough. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label the popover dialog. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe the popover dialog. @default undefined */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired before automatic entry focus is applied. */
  "open-auto-focus": [event: PopoverAutoFocusEvent];
  /** Fired before automatic focus restoration is applied. */
  "close-auto-focus": [event: PopoverAutoFocusEvent];
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];
  /** Fired before an outside pointer-down requests dismissal. */
  "pointer-down-outside": [event: DismissableLayerPointerDownOutsideEvent];
  /** Fired before outside focus movement requests dismissal. */
  "focus-outside": [event: DismissableLayerFocusOutsideEvent];
  /** Fired before outside pointer or focus interaction requests dismissal. */
  "interact-outside": [event: DismissableLayerInteractOutsideEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** Popover content. Receives open state and resolved placement. */
  default(props: PopoverContentSlotState): unknown;
}>();

const context = popoverContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const beforeGuard = useTemplateRef<HTMLSpanElement>("beforeGuard");
const afterGuard = useTemplateRef<HTMLSpanElement>("afterGuard");
const ownerDocument = shallowRef<Document | null>(null);
const present = computed(() => context.open.value || forceMount);
const guarded = computed(() => context.open.value && context.modal.value && trapFocus);
const positionerProps = computed(() =>
  withViewport(
    {
      arrowPadding,
      collisionPadding,
      direction,
      flip,
      hide,
      offset,
      placement,
      reference: context.triggerElement.value,
      safeArea,
      shift,
      size,
      strategy,
      updateOnResize,
      updateOnScroll,
    },
    viewport,
  ),
);

function slotState(value: Placement): PopoverContentSlotState {
  return createPopoverSlotState(value, {
    disabled: context.disabled.value,
    modal: context.modal.value,
    open: context.open.value,
    state: context.state.value,
  });
}

function resolveContentPlacement(value: unknown): Placement {
  return resolvePopoverSlotPlacement(value, placement);
}

const dismissableLayer = createDismissableLayer({
  root: element,
  branches: () =>
    existingPopoverElements(context.triggerElement.value, beforeGuard.value, afterGuard.value),
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape,
  outsideFocus: () => closeOnFocusOutside,
  outsidePointerDown: () => closeOnPointerDownOutside,
  onEscapeKeyDown: (event) => emit("escape-key-down", event),
  onFocusOutside: (event) => emit("focus-outside", event),
  onInteractOutside: (event) => emit("interact-outside", event),
  onPointerDownOutside: (event) => emit("pointer-down-outside", event),
  onDismiss: (event) => {
    emit("dismiss", event);
    context.close(event.originalEvent);
  },
});
const focusGuards = createFocusGuards({
  root: element,
  enabled: guarded,
  fallbackFocus: () => element.value,
});
const focusScope = createFocusScope({
  root: element,
  autoFocus: () => autoFocus,
  contain: guarded,
  restoreFocus: () => restoreFocus,
  initialFocus: () => initialFocus?.(),
  restoreTarget: () => context.triggerElement.value,
  fallbackFocus: () => element.value,
  onMountAutoFocus: (event) => emit("open-auto-focus", event),
  onUnmountAutoFocus: (event) => emit("close-auto-focus", event),
});
const isolation = createInertOutside({
  root: element,
  enabled: () => context.open.value && context.modal.value && inertOutside,
});
const scrollLock = createScrollLock({
  document: ownerDocument,
  enabled: () => context.open.value && context.modal.value && lockScroll,
});

usePopoverContentLifecycle({
  context,
  dismissableLayer,
  element,
  focusGuards,
  focusScope,
  inertOutside: isolation,
  ownerDocument,
  scrollLock,
});

function focusContent(options?: FocusOptions): void {
  element.value?.focus(options);
}

type PopoverContentSetupExpose = Omit<
  PopoverContentExpose,
  "disabled" | "element" | "modal" | "open" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<PopoverState>;
};

const exposed = {
  disabled: context.disabled,
  element,
  focusContent,
  focusFirst: () => focusScope.focusFirst(),
  modal: context.modal,
  open: context.open,
  state: context.state,
} satisfies PopoverContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="popover-content-host"
    part="content-host"
    :hidden="present && context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="context.open.value" :force-mount>
        <Positioner v-bind="positionerProps">
          <template #default="{ placement: positionerPlacement }">
            <span
              v-if="guarded"
              ref="beforeGuard"
              v-bind="focusGuards.beforeProps"
              data-vize-ui="popover-focus-guard"
              part="focus-guard"
            ></span>
            <div
              :id="context.contentId.value"
              ref="element"
              v-bind="dismissableLayer.layerProps"
              role="dialog"
              tabindex="-1"
              :aria-modal="context.modal.value ? 'true' : undefined"
              :aria-label="ariaLabel"
              :aria-labelledby="ariaLabelledby"
              :aria-describedby="ariaDescribedby"
              data-vize-ui="popover-content"
              part="content"
              :hidden="context.open.value ? undefined : true"
              :data-state="context.state.value"
              :data-modal="context.modal.value ? 'true' : 'false'"
              :data-placement="resolveContentPlacement(positionerPlacement)"
              :data-side="popoverSideFromPlacement(resolveContentPlacement(positionerPlacement))"
              :data-align="popoverAlignFromPlacement(resolveContentPlacement(positionerPlacement))"
              :data-top-layer="dismissableLayer.isTopLayer.value ? 'true' : 'false'"
            >
              <slot v-bind="slotState(resolveContentPlacement(positionerPlacement))" />
            </div>
            <span
              v-if="guarded"
              ref="afterGuard"
              v-bind="focusGuards.afterProps"
              data-vize-ui="popover-focus-guard"
              part="focus-guard"
            ></span>
          </template>
        </Positioner>
      </Presence>
    </Portal>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
