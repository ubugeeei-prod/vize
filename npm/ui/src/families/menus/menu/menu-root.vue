<script setup lang="ts">
import { useMenuRoot } from "./menu-root-runtime.ts";
import type { MenuDirection, MenuSlotState } from "./menu-types.ts";

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
   * Initial open state for uncontrolled use.
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
   * Prevent opening and close the menu when it becomes disabled.
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
  /** Trigger and content. Receives the menu open state. */
  default(props: MenuSlotState): unknown;
}>();

const menu = useMenuRoot({
  kind: "menu",
  hint: "menu",
  id: () => id,
  open: () => open,
  defaultOpen: () => defaultOpen,
  modal: () => modal,
  dir: () => dir,
  loop: () => loop,
  disabled: () => disabled,
  onOpenChange: (value, previous, event) => {
    emit("update:open", value);
    emit("open-change", value, previous, event);
  },
});
const slotState = menu.slotState;

defineExpose(menu.expose);
</script>

<template>
  <slot v-bind="slotState" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
