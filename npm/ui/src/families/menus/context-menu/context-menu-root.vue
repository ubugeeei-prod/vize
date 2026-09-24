<script setup lang="ts">
import { computed, shallowRef } from "vue";

import type { VirtualElement } from "../../overlays/positioner/positioner.ts";
import { useMenuRoot } from "../menu/menu-root-runtime.ts";
import type { MenuDirection, MenuEntryFocus, MenuSlotState } from "../menu/menu-types.ts";
import { contextMenuContext } from "./context-menu-context.ts";
import type { ContextMenuPoint, ContextMenuRootExpose } from "./context-menu-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  modal = true,
  dir = "ltr",
  loop = false,
  disabled = false,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use; the menu anchors at the viewport origin
   * until a context-menu request supplies a point.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Make outside content inert and lock document scroll while open.
   *
   * @default true
   */
  readonly modal?: boolean;

  /**
   * Reading direction: flips submenu arrow keys and submenu placement.
   *
   * @default "ltr"
   */
  readonly dir?: MenuDirection;

  /**
   * Wrap arrow-key navigation from the last item to the first and back.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Ignore context-menu requests so the native browser menu shows instead.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired with the requested open value (supports `v-model:open`). */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request with the previous value and native event. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** ContextMenuTrigger and menu content. Receives the menu open state. */
  default(props: MenuSlotState): unknown;
}>();

const anchor = shallowRef<VirtualElement | null>(null);
const disabledState = computed(() => disabled);

function anchorAt(point: ContextMenuPoint): VirtualElement {
  const rect = { x: point.x, y: point.y, width: 0, height: 0 };
  return { getBoundingClientRect: () => rect };
}

const menu = useMenuRoot({
  kind: "context-menu",
  hint: "context-menu",
  id: () => id,
  open: () => open,
  defaultOpen: () => defaultOpen,
  modal: () => modal,
  dir: () => dir,
  loop: () => loop,
  disabled: () => disabled,
  reference: () => anchor.value ?? anchorAt({ x: 0, y: 0 }),
  // Focus returns to whatever held it before the request (the trigger region
  // is not a control and does not own focus).
  restoreTarget: () => null,
  onOpenChange: (value, previous, event) => {
    emit("update:open", value);
    emit("open-change", value, previous, event);
  },
});

function openAt(point: ContextMenuPoint, event: Event | null, entry: MenuEntryFocus): boolean {
  if (disabledState.value) return false;
  anchor.value = anchorAt(point);
  return menu.level.setOpen(true, event, entry);
}

contextMenuContext.provide({ disabled: disabledState, openAt });
const slotState = menu.slotState;

defineExpose({
  ...menu.expose,
  openAt: (point: ContextMenuPoint, event: Event | null = null) => openAt(point, event, "first"),
} satisfies Omit<ContextMenuRootExpose, keyof typeof menu.expose> & typeof menu.expose);
</script>

<template>
  <slot v-bind="slotState" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
