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
import { mentionContext } from "./mention-context.ts";
import type { MentionContentSlotState } from "./mention-types.ts";

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
   * Preferred placement relative to the trigger character before collision handling.
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
   * Gap between the caret line and the popup.
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
   * Accessible name of the suggestion listbox.
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
  /** `MentionItem`s and `MentionEmpty`. Receives open state, placement, and the query. */
  default(props: MentionContentSlotState): unknown;
}>();

const context = mentionContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const positionerProps = computed(() => ({
  collisionPadding,
  flip,
  offset,
  placement,
  reference: context.caretReference,
  shift,
  size,
  strategy,
}));
const listboxProps = computed(() => ({
  role: "listbox" as const,
  onMousedown: keepFieldFocus,
  onPointerdown: keepFieldFocus,
}));

const layer = useDismissableLayer({
  root: element,
  branches: () => (context.fieldElement.value === null ? [] : [context.fieldElement.value]),
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

// Items never take DOM focus: the field keeps it and exposes the highlighted
// item through aria-activedescendant.
function keepFieldFocus(event: MouseEvent | PointerEvent): void {
  if (event.button === 0) event.preventDefault();
}

function isPlacement(value: string): value is Placement {
  return /^(?:top|right|bottom|left)(?:-(?:start|center|end))?$/u.test(value);
}

function placementOf(value: unknown): Placement {
  return typeof value === "string" && isPlacement(value) ? value : placement;
}

function slotState(resolved: unknown): MentionContentSlotState {
  return { open: context.open.value, placement: placementOf(resolved), query: context.query.value };
}

defineExpose({ element });
</script>

<template>
  <div
    data-vize-ui="mention-content-host"
    part="content-host"
    :hidden="context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="context.open.value" :force-mount>
        <Positioner v-bind="positionerProps">
          <template #default="{ placement: resolved }: { placement: Placement }">
            <div
              :id="context.listboxId.value"
              ref="element"
              v-bind="{ ...layer.layerProps, ...listboxProps }"
              tabindex="-1"
              :aria-label="ariaLabel"
              :aria-busy="context.status.value === 'loading' ? 'true' : undefined"
              data-vize-ui="mention-content"
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
