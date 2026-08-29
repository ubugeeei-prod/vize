<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import { collapsibleContext } from "./collapsible-context.ts";
import { useControllableState } from "./controllable-state.ts";
import { deriveDeterministicId, useDeterministicId } from "./deterministic-id.ts";
import type {
  CollapsibleRootExpose,
  CollapsibleSlotState,
  CollapsibleState,
} from "./collapsible-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
} = defineProps<{
  /**
   * Consumer-owned Collapsible base id. `null` and `undefined` select a deterministic fallback.
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
   * Disable trigger activation while preserving the current disclosure state.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the Collapsible requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Collapsible children. Receives the current open and disabled state. */
  default(props: CollapsibleSlotState): unknown;
}>();

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const isOpen = openState.value;
const disabledState = computed(() => disabled);
const state = computed<CollapsibleState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "collapsible" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const slotState = computed<CollapsibleSlotState>(() => ({
  disabled: disabledState.value,
  open: isOpen.value,
  state: state.value,
}));

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = isOpen.value;
  const changed = openState.set(value);
  if (changed) {
    emit("update:open", value);
    emit("open-change", value, previous, nativeEvent);
  }
  return changed;
}

const context = collapsibleContext.provide({
  id: baseId,
  triggerId,
  contentId,
  open: isOpen,
  disabled: disabledState,
  state,
  setOpen,
  expand: (nativeEvent = null) => setOpen(true, nativeEvent),
  collapse: (nativeEvent = null) => setOpen(false, nativeEvent),
  toggle: (nativeEvent = null) => setOpen(!isOpen.value, nativeEvent),
});

type CollapsibleRootSetupExpose = Omit<
  CollapsibleRootExpose,
  "contentId" | "disabled" | "id" | "open" | "state" | "triggerId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<CollapsibleState>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  collapse: context.collapse,
  contentId,
  disabled: disabledState,
  expand: context.expand,
  id: baseId,
  open: isOpen,
  setOpen,
  state,
  toggle: context.toggle,
  triggerId,
} satisfies CollapsibleRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="collapsible-root"
    part="root"
    :data-state="state"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
