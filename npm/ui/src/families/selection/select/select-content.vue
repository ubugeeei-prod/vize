<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import { useDismissableLayer } from "../../overlays/dismissable-layer/dismissable-layer.ts";
import Portal from "../../overlays/portal/portal.vue";
import Positioner from "../../overlays/positioner/positioner.vue";
import type { Placement, PositionerStrategy } from "../../overlays/positioner/positioner.ts";
import Presence from "../../overlays/presence/presence.vue";
import { selectContext } from "./select-context.ts";
import { createItemAlignedReference } from "./select-positioning.ts";
import type {
  SelectContentExpose,
  SelectContentSlotState,
  SelectDismissEvent,
  SelectEscapeKeyDownEvent,
  SelectPointerDownOutsideEvent,
  SelectPosition,
} from "./select-types.ts";

const {
  position = "popper",
  placement = "bottom-start",
  strategy = "fixed",
  offset = 4,
  collisionPadding = 8,
  flip = true,
  shift = true,
  size = true,
  hide = true,
  forceMount = false,
  to = "body",
  portalDisabled = false,
  defer = true,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * `popper` anchors below the trigger; `item-aligned` overlays the selected option on the trigger.
   *
   * @default "popper"
   */
  readonly position?: SelectPosition;

  /**
   * Preferred popper placement before collision handling.
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
   * Gap between trigger and listbox in popper mode.
   *
   * @default 4
   */
  readonly offset?: number;

  /**
   * Viewport padding the listbox should not cross.
   *
   * @default 8
   */
  readonly collisionPadding?: number;

  /**
   * Flip to the opposite side when the preferred side overflows more (popper mode).
   *
   * @default true
   */
  readonly flip?: boolean;

  /**
   * Shift the listbox back into the viewport.
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
   * Hide when the trigger scrolls out of view.
   *
   * @default true
   */
  readonly hide?: boolean;

  /**
   * Keep options mounted (hidden) while closed so labels and typeahead work before first open.
   *
   * @default false
   */
  readonly forceMount?: boolean;

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
   * Let Escape close the popup.
   *
   * @default true
   */
  readonly closeOnEscape?: boolean;

  /**
   * Let an outside pointer-down close the popup.
   *
   * @default true
   */
  readonly closeOnPointerDownOutside?: boolean;

  /**
   * Accessible name for the listbox when the trigger label is not enough.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids labelling the listbox. Defaults to the trigger.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired before Escape closes the popup. Call `preventDefault()` to keep it open. */
  "escape-key-down": [event: SelectEscapeKeyDownEvent];

  /** Fired before an outside pointer-down closes the popup. Call `preventDefault()` to keep it open. */
  "pointer-down-outside": [event: SelectPointerDownOutsideEvent];

  /** Fired after an unprevented dismissal request closes the popup. */
  dismiss: [event: SelectDismissEvent];
}>();

defineSlots<{
  /** Options, groups, separators, viewport, and scroll buttons. Receives open state and placement. */
  default(props: SelectContentSlotState): unknown;
}>();

const context = selectContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const itemAligned = computed(() => position === "item-aligned");
const itemAlignedReference = createItemAlignedReference({
  anchor: () => context.anchorElement(),
  content: () => element.value,
  trigger: () => context.triggerElement.value,
});
const positionerProps = computed(() => ({
  collisionPadding,
  direction: context.direction.value,
  flip: itemAligned.value ? false : flip,
  hide,
  offset: itemAligned.value ? 0 : offset,
  placement: itemAligned.value ? ("bottom-start" as const) : placement,
  reference: itemAligned.value ? itemAlignedReference : context.referenceElement.value,
  shift,
  size,
  strategy,
}));
const labelledby = computed(() =>
  ariaLabel === undefined ? (ariaLabelledby ?? context.listboxLabelledby.value) : ariaLabelledby,
);
const listboxProps = computed<{
  readonly role: "listbox";
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
}>(() => ({
  role: "listbox",
  onMousedown: keepTriggerFocus,
  onPointerdown: keepTriggerFocus,
}));

const layer = useDismissableLayer({
  root: element,
  branches: () => context.dismissBranches.value,
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape,
  outsideFocus: () => true,
  outsidePointerDown: () => closeOnPointerDownOutside,
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

function placementOf(value: unknown): Placement {
  return typeof value === "string" && isPlacement(value) ? value : placement;
}

function isPlacement(value: string): value is Placement {
  return /^(?:top|right|bottom|left)(?:-(?:start|center|end))?$/u.test(value);
}

function slotState(resolved: unknown): SelectContentSlotState {
  return { open: context.open.value, placement: placementOf(resolved), position };
}

type SelectContentSetupExpose = Omit<SelectContentExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SelectContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :data-vize-ui="`${context.partPrefix}-content-host`"
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
              :aria-labelledby="labelledby"
              :aria-multiselectable="context.multiple.value ? 'true' : undefined"
              :aria-required="context.required.value ? 'true' : undefined"
              :data-vize-ui="`${context.partPrefix}-content`"
              part="content"
              :hidden="context.open.value ? undefined : true"
              :data-state="context.state.value"
              :data-position="position"
              :data-placement="placementOf(resolved)"
              :dir="context.direction.value"
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
