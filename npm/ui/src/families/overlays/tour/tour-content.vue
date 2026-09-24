<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef, watch, watchEffect } from "vue";
import type { ComputedRef } from "vue";

import { createFocusScope } from "../../accessibility/focus-scope/focus-scope.ts";
import type { FocusScopeAutoFocusEvent } from "../../accessibility/focus-scope/focus-scope.ts";
import { createDismissableLayer } from "../dismissable-layer/dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../dismissable-layer/dismissable-layer.ts";
import Portal from "../portal/portal.vue";
import { positionerContext } from "../positioner/positioner-context.ts";
import { usePositioner } from "../positioner/positioner.ts";
import type { PositionerStrategy, Rect } from "../positioner/positioner.ts";
import Presence from "../presence/presence.vue";
import { tourContext } from "./tour-context.ts";
import {
  isTourEditableTarget,
  tourAlignFromPlacement,
  tourSideFromPlacement,
} from "./tour-state.ts";
import type {
  TourContentExpose,
  TourContentPlacement,
  TourContentSlotState,
  TourState,
} from "./tour-types.ts";

const {
  forceMount = false,
  to = "body",
  portalDisabled = false,
  defer = true,
  strategy = "fixed",
  offset = 12,
  collisionPadding = 8,
  arrowPadding = 0,
  flip = true,
  shift = true,
  size = false,
  safeArea = false,
  hide = false,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
  modal = false,
  autoFocus = true,
  restoreFocus = true,
  focusOnStepChange = true,
  closeOnEscape = true,
  closeOnPointerDownOutside = false,
  closeOnFocusOutside = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Keep the content mounted while the tour is closed. @default false */
  readonly forceMount?: boolean;
  /** CSS selector or element the tour layer is moved into. @default "body" */
  readonly to?: string | HTMLElement;
  /** Render in place instead of teleporting. @default false */
  readonly portalDisabled?: boolean;
  /** Keep content in place until the portal target exists, avoiding SSR mismatch. @default true */
  readonly defer?: boolean;
  /** CSS positioning mode published on the floating host. @default "fixed" */
  readonly strategy?: PositionerStrategy;
  /** Gap on the main axis between the target and the content. @default 12 */
  readonly offset?: number;
  /** Viewport padding the content should not cross. @default 8 */
  readonly collisionPadding?: number;
  /** Inset kept between the arrow and content edges. @default 0 */
  readonly arrowPadding?: number;
  /** Flip to the opposite side when the preferred side overflows more. @default true */
  readonly flip?: boolean;
  /** Shift the content back into the viewport after flip. @default true */
  readonly shift?: boolean;
  /** Constrain the host and publish positioner available-size CSS variables. @default false */
  readonly size?: boolean;
  /** Keep content clear of safe-area insets. @default false */
  readonly safeArea?: boolean;
  /** Mark the host hidden when the target leaves the viewport. @default false */
  readonly hide?: boolean;
  /** Recalculate while ancestors scroll. @default true */
  readonly updateOnScroll?: boolean;
  /** Recalculate when the document or visual viewport resizes. @default true */
  readonly updateOnResize?: boolean;
  /** Viewport used for flip, shift, and hide. @default undefined */
  readonly viewport?: Rect;
  /** Publish `aria-modal="true"` and contain focus inside the content. @default false */
  readonly modal?: boolean;
  /** Move focus to the content when the tour opens. @default true */
  readonly autoFocus?: boolean;
  /** Restore focus to the previously focused element when the tour closes. @default true */
  readonly restoreFocus?: boolean;
  /** Move focus to the content whenever the current step changes. @default true */
  readonly focusOnStepChange?: boolean;
  /** Let Escape dismiss the tour with reason `escape-key`. @default true */
  readonly closeOnEscape?: boolean;
  /** Let pointer-down outside the content and target dismiss the tour. @default false */
  readonly closeOnPointerDownOutside?: boolean;
  /** Let focus moving outside the content and target dismiss the tour. @default false */
  readonly closeOnFocusOutside?: boolean;
  /** Accessible name when no TourTitle labels the content. @default undefined */
  readonly ariaLabel?: string;
  /** Ids that label the content. `null` omits the default TourTitle id. @default undefined */
  readonly ariaLabelledby?: string | null;
  /** Ids that describe the content. `null` omits the default TourDescription id. @default undefined */
  readonly ariaDescribedby?: string | null;
}>();

const emit = defineEmits<{
  /** Fired before automatic entry focus is applied. */
  "open-auto-focus": [event: FocusScopeAutoFocusEvent];
  /** Fired before automatic focus restoration is applied. */
  "close-auto-focus": [event: FocusScopeAutoFocusEvent];
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];
  /** Fired before an outside pointer-down requests dismissal. */
  "pointer-down-outside": [event: DismissableLayerPointerDownOutsideEvent];
  /** Fired before outside focus movement requests dismissal. */
  "focus-outside": [event: DismissableLayerFocusOutsideEvent];
  /** Fired before outside pointer or focus interaction requests dismissal. */
  "interact-outside": [event: DismissableLayerInteractOutsideEvent];
  /** Fired after an unprevented dismissal request, before the tour closes. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** Step content. Receives the current step, progress, and resolved placement. */
  default(props: TourContentSlotState): unknown;
}>();

const context = tourContext.use();
const floating = useTemplateRef<HTMLDivElement>("floating");
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const anchored = computed(() => context.targetState.value === "resolved");
const positioner = usePositioner({
  arrowPadding: () => arrowPadding,
  collisionPadding: () => collisionPadding,
  direction: () => context.dir.value,
  flip: () => flip,
  hide: () => hide,
  offset: () => offset,
  placement: () => context.placement.value,
  safeArea: () => safeArea,
  shift: () => shift,
  size: () => size,
  strategy: () => strategy,
  updateOnResize: () => updateOnResize,
  updateOnScroll: () => updateOnScroll,
  viewport: () => viewport,
});
positionerContext.provide(positioner);

const placement = computed<TourContentPlacement>(() =>
  anchored.value ? positioner.resolvedPlacement.value : "center",
);
const labelledBy = computed(() => {
  if (ariaLabel !== undefined && ariaLabelledby === undefined) return undefined;
  return ariaLabelledby === null ? undefined : (ariaLabelledby ?? context.titleId.value);
});
const describedBy = computed(() =>
  ariaDescribedby === null ? undefined : (ariaDescribedby ?? context.descriptionId.value),
);
const slotState = computed<TourContentSlotState>(() => ({
  ...context.slotState.value,
  align: tourAlignFromPlacement(placement.value),
  placement: placement.value,
  side: tourSideFromPlacement(placement.value),
}));
let mounted = false;

const dismissableLayer = createDismissableLayer({
  root: element,
  branches: () => {
    const target = context.target.value;
    return target === null ? [] : [target];
  },
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
    context.dismiss(event.reason, event.originalEvent);
  },
});
const focusScope = createFocusScope({
  root: element,
  autoFocus: () => autoFocus,
  contain: () => modal && context.open.value,
  restoreFocus: () => restoreFocus,
  initialFocus: () => element.value,
  fallbackFocus: () => element.value,
  onMountAutoFocus: (event) => emit("open-auto-focus", event),
  onUnmountAutoFocus: (event) => emit("close-auto-focus", event),
});

function syncControllers(): void {
  if (mounted && context.open.value && element.value) {
    dismissableLayer.activate();
    focusScope.activate();
    return;
  }
  dismissableLayer.deactivate();
  focusScope.deactivate();
}

function syncPositioner(): void {
  if (!mounted) return;
  positioner.setFloating(floating.value);
  positioner.setReference(anchored.value ? context.target.value : null);
}

watch(element, syncControllers, { flush: "post" });
watch(() => context.open.value, syncControllers, { flush: "post" });
watch([floating, anchored, () => context.target.value], syncPositioner, { flush: "post" });
watch(
  () => context.step.value?.value,
  () => {
    if (!mounted || !focusOnStepChange || !context.open.value) return;
    element.value?.focus({ preventScroll: true });
  },
  { flush: "post" },
);

// Placement geometry is measurement output applied imperatively, like Positioner:
// the authoring gate keeps `:style` bindings out of templates.
watchEffect(
  () => {
    if (floating.value) floating.value.style.cssText = anchored.value ? positioner.style.value : "";
  },
  { flush: "sync" },
);

// Arrow navigation is bound imperatively: the dialog is a non-interactive container whose
// descendants own activation, so no template event handler is attached to it.
watch(
  element,
  (next, _previous, onCleanup) => {
    if (!next) return;
    next.addEventListener("keydown", onKeydown);
    onCleanup(() => next.removeEventListener("keydown", onKeydown));
  },
  { flush: "post" },
);

onMounted(() => {
  mounted = true;
  syncControllers();
  syncPositioner();
});

onUnmounted(() => {
  mounted = false;
  dismissableLayer.deactivate();
  focusScope.deactivate();
  dismissableLayer.dispose();
  focusScope.dispose();
});

function onKeydown(event: KeyboardEvent): void {
  if (!context.keyboardNavigation.value || event.defaultPrevented) return;
  if (event.altKey || event.ctrlKey || event.metaKey || isTourEditableTarget(event.target)) return;
  if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
  const forward = (event.key === "ArrowRight") === (context.dir.value === "ltr");
  const moved = forward ? context.next(event) : context.previous(event);
  if (moved) event.preventDefault();
}

function focusContent(options?: FocusOptions): void {
  element.value?.focus(options);
}

function update(): void {
  if (anchored.value) positioner.update();
}

type TourContentSetupExpose = Omit<TourContentExpose, "element" | "placement" | "state"> & {
  readonly element: typeof element;
  readonly placement: ComputedRef<TourContentPlacement>;
  readonly state: ComputedRef<TourState>;
};

const exposed = {
  element,
  focusContent,
  placement,
  state: context.state,
  update,
} satisfies TourContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="tour-content-host"
    part="content-host"
    :hidden="present && context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="context.open.value" :force-mount>
        <div
          ref="floating"
          data-vize-ui="tour-positioner"
          part="positioner"
          :data-placement="placement"
          :data-vize-positioner-ready="positioner.ready.value ? 'true' : 'false'"
          :data-vize-hidden="anchored && positioner.hidden.value ? 'true' : undefined"
        >
          <div
            :id="context.contentId.value"
            ref="element"
            v-bind="dismissableLayer.layerProps"
            role="dialog"
            tabindex="-1"
            :aria-modal="modal ? 'true' : undefined"
            :aria-label="ariaLabel"
            :aria-labelledby="labelledBy"
            :aria-describedby="describedBy"
            data-vize-ui="tour-content"
            part="content"
            :hidden="context.open.value ? undefined : true"
            :data-state="context.state.value"
            :data-step="context.step.value?.value"
            :data-target="context.targetState.value"
            :data-placement="placement"
            :data-side="tourSideFromPlacement(placement)"
            :data-align="tourAlignFromPlacement(placement)"
            :data-first="context.first.value ? 'true' : undefined"
            :data-last="context.last.value ? 'true' : undefined"
          >
            <slot v-bind="slotState" />
          </div>
        </div>
      </Presence>
    </Portal>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
