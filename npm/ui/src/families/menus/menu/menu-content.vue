<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import Portal from "../../overlays/portal/portal.vue";
import Positioner from "../../overlays/positioner/positioner.vue";
import type { Placement, PositionerStrategy, Rect } from "../../overlays/positioner/positioner.ts";
import Presence from "../../overlays/presence/presence.vue";
import { menuContentContext, menuLevelContext, menuTreeContext } from "./menu-context.ts";
import { useMenuContent } from "./menu-content-runtime.ts";
import { alignOf, resolvePlacement, sideOf } from "./menu-dom.ts";
import type {
  MenuAutoFocusEvent,
  MenuContentExpose,
  MenuContentSlotState,
  MenuDismissEvent,
  MenuEscapeKeyDownEvent,
  MenuFocusOutsideEvent,
  MenuInteractOutsideEvent,
  MenuPointerDownOutsideEvent,
} from "./menu-types.ts";

const {
  forceMount = false,
  to = "body",
  portalDisabled = false,
  defer = true,
  placement = "bottom-start",
  strategy = "fixed",
  offset = 4,
  collisionPadding = 4,
  arrowPadding = 0,
  flip = true,
  shift = true,
  size = false,
  safeArea = false,
  hide = true,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  closeOnFocusOutside = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /** Keep the content mounted while the menu is closed. @default false */
  readonly forceMount?: boolean;
  /** CSS selector or element the menu layer is moved into. @default "body" */
  readonly to?: string | HTMLElement;
  /** Render in place instead of teleporting. @default false */
  readonly portalDisabled?: boolean;
  /** Keep content in place until the target exists, avoiding SSR mismatch. @default true */
  readonly defer?: boolean;
  /** Preferred placement before collision handling. @default "bottom-start" */
  readonly placement?: Placement;
  /** CSS positioning mode published on the floating host. @default "fixed" */
  readonly strategy?: PositionerStrategy;
  /** Gap on the main axis between the reference and content. @default 4 */
  readonly offset?: number;
  /** Viewport padding the content should not cross. @default 4 */
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
  /** Hide when the reference no longer intersects the viewport. @default true */
  readonly hide?: boolean;
  /** Recalculate while ancestors scroll. @default true */
  readonly updateOnScroll?: boolean;
  /** Recalculate when the viewport resizes. @default true */
  readonly updateOnResize?: boolean;
  /** Viewport used for flip, shift, and hide. @default undefined */
  readonly viewport?: Rect;
  /** Let Escape close the menu. @default true */
  readonly closeOnEscape?: boolean;
  /** Let outside pointer-down close the menu. @default true */
  readonly closeOnPointerDownOutside?: boolean;
  /** Let outside focus movement close the menu. @default true */
  readonly closeOnFocusOutside?: boolean;
  /** Accessible name when the trigger does not label the menu. @default undefined */
  readonly ariaLabel?: string;
  /** Ids that label the menu; defaults to the trigger id when a trigger exists. @default undefined */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired before automatic entry focus; preventable. */
  "open-auto-focus": [event: MenuAutoFocusEvent];
  /** Fired before focus returns to the trigger; preventable. */
  "close-auto-focus": [event: MenuAutoFocusEvent];
  /** Fired before Escape closes the menu; preventable. */
  "escape-key-down": [event: MenuEscapeKeyDownEvent];
  /** Fired before an outside pointer-down closes the menu; preventable. */
  "pointer-down-outside": [event: MenuPointerDownOutsideEvent];
  /** Fired before outside focus closes the menu; preventable. */
  "focus-outside": [event: MenuFocusOutsideEvent];
  /** Fired before any outside interaction closes the menu; preventable. */
  "interact-outside": [event: MenuInteractOutsideEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: MenuDismissEvent];
}>();

defineSlots<{
  /** Menu items. Receives open state and the resolved placement. */
  default(props: MenuContentSlotState): unknown;
}>();

const tree = menuTreeContext.use();
const level = menuLevelContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => level.open.value || forceMount);
const menu = useMenuContent({
  element,
  tree,
  level,
  parentContent: menuContentContext.useOptional(),
  closeOnEscape: () => closeOnEscape,
  closeOnPointerDownOutside: () => closeOnPointerDownOutside,
  closeOnFocusOutside: () => closeOnFocusOutside,
  callbacks: {
    openAutoFocus: (event) => emit("open-auto-focus", event),
    closeAutoFocus: (event) => emit("close-auto-focus", event),
    escapeKeyDown: (event) => emit("escape-key-down", event),
    pointerDownOutside: (event) => emit("pointer-down-outside", event),
    focusOutside: (event) => emit("focus-outside", event),
    interactOutside: (event) => emit("interact-outside", event),
    dismiss: (event) => emit("dismiss", event),
  },
});
const labelledby = computed(
  () =>
    ariaLabelledby ?? (level.hasTrigger.value && !ariaLabel ? level.triggerId.value : undefined),
);
const positionerProps = computed(() => ({
  arrowPadding,
  collisionPadding,
  direction: tree.dir.value,
  flip,
  hide,
  offset,
  placement,
  reference: level.reference.value,
  safeArea,
  shift,
  size,
  strategy,
  updateOnResize,
  updateOnScroll,
  ...(viewport === undefined ? {} : { viewport }),
}));

function place(value: unknown): Placement {
  return resolvePlacement(value, placement);
}

function slotState(value: Placement): MenuContentSlotState {
  return {
    align: alignOf(value),
    dir: tree.dir.value,
    modal: tree.modal.value,
    open: level.open.value,
    placement: value,
    side: sideOf(value),
    state: level.state.value,
  };
}

const exposed = {
  element,
  focusContent: menu.focusContent,
  focusFirst: menu.focusFirst,
  focusLast: menu.focusLast,
  open: level.open,
  state: level.state,
} satisfies Omit<MenuContentExpose, "element" | "open" | "state"> & {
  readonly element: typeof element;
  readonly open: typeof level.open;
  readonly state: typeof level.state;
};

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="menu-content-host"
    part="content-host"
    :hidden="present && level.open.value ? undefined : true"
    :data-state="level.state.value"
  >
    <Portal v-if="present" :to :disabled="portalDisabled" :defer>
      <Presence :present="level.open.value" :force-mount>
        <Positioner v-bind="positionerProps">
          <template #default="{ placement: resolved }: { placement: Placement }">
            <div
              :id="level.contentId.value"
              ref="element"
              v-bind="menu.contentProps"
              :dir="tree.dir.value"
              :aria-label="ariaLabel"
              :aria-labelledby="labelledby"
              data-vize-ui="menu-content"
              :data-menu-kind="tree.kind"
              part="content"
              :hidden="level.open.value ? undefined : true"
              :data-state="level.state.value"
              :data-modal="tree.modal.value ? 'true' : 'false'"
              :data-placement="place(resolved)"
              :data-side="sideOf(place(resolved))"
              :data-align="alignOf(place(resolved))"
              :data-top-layer="menu.dismissableLayer.isTopLayer.value ? 'true' : 'false'"
            >
              <slot v-bind="slotState(place(resolved))" />
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
