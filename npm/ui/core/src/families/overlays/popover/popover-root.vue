<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { deriveDeterministicId, useDeterministicId } from "../../../deterministic-id.ts";
import { popoverContext } from "./popover-context.ts";
import type { PopoverRootExpose, PopoverSlotState, PopoverState } from "./popover-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  modal = false,
  disabled = false,
} = defineProps<{
  /**
   * Consumer-owned Popover base id. `null` and `undefined` select a deterministic fallback.
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
   * Whether the popover makes outside content inert and focus-contained while open.
   *
   * @default false
   */
  readonly modal?: boolean;

  /**
   * Disable trigger-driven opening and request closure when already open.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the Popover requests a controlled open value. */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Popover children. Receives the current open, modal, and disabled state. */
  default(props: PopoverSlotState): unknown;
}>();

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const disabledState = computed(() => disabled);
const isOpen = computed(() => openState.value.value && !disabledState.value);
const modalState = computed(() => modal);
const state = computed<PopoverState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "popover" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const slotState = computed<PopoverSlotState>(() => ({
  disabled: disabledState.value,
  modal: modalState.value,
  open: isOpen.value,
  state: state.value,
}));
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const contentElement = shallowRef<HTMLDivElement | null>(null);

function commitOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = openState.value.value && !disabledState.value;
  const next = value && !disabledState.value;
  const changed = openState.set(next);
  if (changed || previous !== next) {
    emit("update:open", next);
    emit("open-change", next, previous, nativeEvent);
  }
  return changed || previous !== next;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  return commitOpen(value, nativeEvent);
}

function openPopover(nativeEvent: Event | null = null): boolean {
  return commitOpen(true, nativeEvent);
}

function close(nativeEvent: Event | null = null): boolean {
  return commitOpen(false, nativeEvent);
}

function toggle(nativeEvent: Event | null = null): boolean {
  return commitOpen(!isOpen.value, nativeEvent);
}

watch(disabledState, (next, previousDisabled) => {
  if (next && openState.value.value) {
    const previous = !previousDisabled && openState.value.value;
    const changed = openState.set(false);
    if (changed || previous) {
      emit("update:open", false);
      emit("open-change", false, previous, null);
    }
  }
});

const context = popoverContext.provide({
  id: baseId,
  triggerId,
  contentId,
  open: isOpen,
  modal: modalState,
  disabled: disabledState,
  state,
  triggerElement,
  contentElement,
  setOpen,
  openPopover,
  close,
  toggle,
});

type PopoverRootSetupExpose = Omit<
  PopoverRootExpose,
  "contentId" | "disabled" | "id" | "modal" | "open" | "state" | "triggerId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<PopoverState>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  close,
  contentId,
  disabled: disabledState,
  id: baseId,
  modal: modalState,
  open: isOpen,
  openPopover,
  setOpen,
  state,
  toggle,
  triggerId,
} satisfies PopoverRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="popover-root"
    part="root"
    :data-state="state"
    :data-modal="modalState ? 'true' : 'false'"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
