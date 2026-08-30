<script setup lang="ts">
import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { dialogContext } from "./dialog-context.ts";
import type { DialogRootExpose, DialogSlotState, DialogState } from "./dialog-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  modal = true,
} = defineProps<{
  /**
   * Consumer-owned Dialog base id. `null` and `undefined` select a deterministic fallback.
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
   * Whether the dialog makes outside content inert, focus-contained, and scroll-locked.
   *
   * @default true
   */
  readonly modal?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the Dialog requests a controlled open value. */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Dialog children. Receives the current open and modal state. */
  default(props: DialogSlotState): unknown;
}>();

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const isOpen = openState.value;
const modalState = computed(() => modal);
const state = computed<DialogState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "dialog" });
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const slotState = computed<DialogSlotState>(() => ({
  modal: modalState.value,
  open: isOpen.value,
  state: state.value,
}));
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const overlayElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLDivElement | null>(null);

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = isOpen.value;
  const changed = openState.set(value);
  if (changed) {
    emit("update:open", value);
    emit("open-change", value, previous, nativeEvent);
  }
  return changed;
}

const context = dialogContext.provide({
  id: baseId,
  contentId,
  titleId,
  descriptionId,
  open: isOpen,
  modal: modalState,
  state,
  triggerElement,
  overlayElement,
  contentElement,
  setOpen,
  openDialog: (nativeEvent = null) => setOpen(true, nativeEvent),
  close: (nativeEvent = null) => setOpen(false, nativeEvent),
  toggle: (nativeEvent = null) => setOpen(!isOpen.value, nativeEvent),
});

type DialogRootSetupExpose = Omit<
  DialogRootExpose,
  "contentId" | "descriptionId" | "id" | "modal" | "open" | "state" | "titleId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly id: ComputedRef<string>;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<DialogState>;
  readonly titleId: ComputedRef<string>;
};

const exposed = {
  close: context.close,
  contentId,
  descriptionId,
  id: baseId,
  modal: modalState,
  open: isOpen,
  openDialog: context.openDialog,
  setOpen,
  state,
  titleId,
  toggle: context.toggle,
} satisfies DialogRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="dialog-root"
    part="root"
    :data-state="state"
    :data-modal="modal ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
