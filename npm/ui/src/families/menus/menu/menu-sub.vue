<script setup lang="ts">
import { computed } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { menuLevelContext } from "./menu-context.ts";
import { createMenuLevel } from "./menu-root-runtime.ts";
import type { MenuSubExpose, MenuSubSlotState } from "./menu-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
} = defineProps<{
  /**
   * Consumer-owned submenu base id. `null` and `undefined` select a deterministic fallback.
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
   * Initial open state for uncontrolled use (only honored while the parent is open).
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Prevent the submenu from opening; its trigger stays focusable but inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired with the requested submenu open value (supports `v-model:open`). */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request with the previous value and native event. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** MenuSubTrigger and MenuSubContent. Receives the submenu open state. */
  default(props: MenuSubSlotState): unknown;
}>();

const parent = menuLevelContext.use();
const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const baseId = useDeterministicId({ id: () => id, hint: "menu-sub" });
const level = createMenuLevel({
  parent,
  baseId,
  requestedOpen: computed(() => openState.value.value),
  commitOpen: (value) => openState.set(value),
  disabled: () => disabled,
  onOpenChange: (value, previous, event) => {
    emit("update:open", value);
    emit("open-change", value, previous, event);
  },
});
menuLevelContext.provide(level);

defineExpose({
  contentId: level.contentId,
  open: level.open,
  setOpen: (value: boolean, event: Event | null = null) => level.setOpen(value, event, "first"),
  state: level.state,
  triggerId: level.triggerId,
} satisfies Omit<MenuSubExpose, "contentId" | "open" | "state" | "triggerId"> & {
  readonly contentId: typeof level.contentId;
  readonly open: typeof level.open;
  readonly state: typeof level.state;
  readonly triggerId: typeof level.triggerId;
});
</script>

<template>
  <slot :open="level.open.value" :state="level.state.value" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
