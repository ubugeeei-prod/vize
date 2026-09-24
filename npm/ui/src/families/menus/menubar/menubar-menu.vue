<script setup lang="ts">
import { computed } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useMenuRoot } from "../menu/menu-root-runtime.ts";
import { menubarContext, menubarMenuContext } from "./menubar-context.ts";
import type { MenubarMenuExpose, MenubarMenuSlotState } from "./menubar-types.ts";

const {
  value = undefined,
  modal = false,
  loop = false,
  disabled = false,
} = defineProps<{
  /**
   * Stable value identifying this menu in the menubar `v-model`.
   * `undefined` selects a deterministic fallback.
   *
   * @default undefined
   */
  readonly value?: string;

  /**
   * Make outside content inert while this menu is open. Menubars default to
   * non-modal so hovering sibling triggers can switch menus.
   *
   * @default false
   */
  readonly modal?: boolean;

  /**
   * Wrap Up/Down navigation inside this menu.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Prevent this menu from opening; its trigger stays focusable per WAI-ARIA APG.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** MenubarTrigger and menu content. Receives this menu's open state and value. */
  default(props: MenubarMenuSlotState): unknown;
}>();

const menubar = menubarContext.use();
const fallback = useDeterministicId({ hint: "menubar-menu" });
const menuValue = computed(() => value ?? fallback.value);

const menu = useMenuRoot({
  kind: "menubar",
  hint: "menubar-menu",
  id: () => undefined,
  open: () => menubar.value.value === menuValue.value,
  defaultOpen: () => false,
  modal: () => modal,
  dir: () => menubar.dir.value,
  loop: () => loop,
  disabled: () => disabled,
  edgeNavigate: (direction, event) => menubar.move(menuValue.value, direction, event),
  onOpenChange: (open, _previous, event) => {
    if (open) menubar.setValue(menuValue.value, event);
    else if (menubar.value.value === menuValue.value) menubar.setValue(null, event);
  },
});
menubarMenuContext.provide({ value: menuValue });

const slotState = computed<MenubarMenuSlotState>(() => ({
  ...menu.slotState.value,
  value: menuValue.value,
}));

defineExpose({
  open: menu.level.open,
  setOpen: (next: boolean) => menu.level.setOpen(next, null, "first"),
  value: menuValue,
} satisfies Omit<MenubarMenuExpose, "open" | "value"> & {
  readonly open: typeof menu.level.open;
  readonly value: typeof menuValue;
});
</script>

<template>
  <slot v-bind="slotState" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
