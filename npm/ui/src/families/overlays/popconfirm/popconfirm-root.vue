<script setup lang="ts">
import { computed, onScopeDispose, shallowReadonly, shallowRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { PopoverRoot } from "../popover/popover.ts";
import { popconfirmContext } from "./popconfirm-context.ts";
import type {
  PopconfirmCancelReason,
  PopconfirmConfirmHandler,
  PopconfirmRootExpose,
  PopconfirmSlotState,
  PopconfirmState,
} from "./popconfirm-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
  onConfirm = undefined,
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
   * Disable the trigger and close an open confirmation.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Confirmation handler (bind with `@confirm`). A returned promise keeps the
   * confirmation open and pending; it closes on resolve and stays open on reject.
   *
   * @default undefined
   */
  readonly onConfirm?: PopconfirmConfirmHandler;
}>();

const emit = defineEmits<{
  /** Fired when the Popconfirm requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];

  /** Fired after the confirmation handler succeeded and the popover closed. */
  confirmed: [nativeEvent: Event | null];

  /** Fired when the confirmation closed without confirming. */
  cancel: [reason: PopconfirmCancelReason, nativeEvent: Event | null];

  /** Fired when the confirmation handler threw or rejected; the popover stays open. */
  error: [reason: unknown];
}>();

defineSlots<{
  /** Trigger and content children. Receives the confirmation state. */
  default?(props: PopconfirmSlotState): unknown;
}>();

const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const isOpen = computed(() => openState.value.value && !disabled);
const pendingState = shallowRef(false);
const pending = computed(() => pendingState.value);
const errorState = shallowRef<unknown>(null);
const state = computed<PopconfirmState>(() => (pending.value ? "pending" : "idle"));
const baseId = useDeterministicId({ id: () => id, hint: "popconfirm" });
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const confirmId = computed(() => deriveDeterministicId(baseId.value, "confirm"));
const cancelId = computed(() => deriveDeterministicId(baseId.value, "cancel"));
const confirmElement = shallowRef<HTMLButtonElement | null>(null);
const cancelElement = shallowRef<HTMLButtonElement | null>(null);
const slotState = computed<PopconfirmSlotState>(() => ({
  error: errorState.value,
  open: isOpen.value,
  pending: pending.value,
  state: state.value,
}));
let generation = 0;
let disposed = false;

onScopeDispose(() => {
  disposed = true;
});

function readOpen(): boolean {
  return isOpen.value;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  if (!value && pendingState.value) return false;
  const previous = readOpen();
  if (!openState.set(value)) return false;
  if (value) errorState.value = null;
  emit("update:open", value);
  emit("open-change", value, previous, nativeEvent);
  return true;
}

function cancel(nativeEvent: Event | null, reason: PopconfirmCancelReason): boolean {
  if (pendingState.value || !readOpen()) return false;
  emit("cancel", reason, nativeEvent);
  return setOpen(false, nativeEvent);
}

function fail(reason: unknown): false {
  errorState.value = reason;
  emit("error", reason);
  return false;
}

function succeed(nativeEvent: Event | null): true {
  errorState.value = null;
  setOpen(false, nativeEvent);
  emit("confirmed", nativeEvent);
  return true;
}

type HandlerInvocation =
  | { readonly ok: true; readonly value: void | PromiseLike<void> }
  | { readonly ok: false; readonly reason: unknown };

function invokeHandler(): HandlerInvocation {
  try {
    return { ok: true, value: onConfirm?.() };
  } catch (reason) {
    return { ok: false, reason };
  }
}

async function confirm(nativeEvent: Event | null = null): Promise<boolean> {
  if (pendingState.value || !readOpen()) return false;
  const invocation = invokeHandler();
  if (!invocation.ok) return fail(invocation.reason);
  const result = invocation.value;
  if (result === undefined) return succeed(nativeEvent);
  const current = ++generation;
  pendingState.value = true;
  try {
    await result;
    if (disposed || current !== generation) return false;
    pendingState.value = false;
    return succeed(nativeEvent);
  } catch (reason) {
    if (disposed || current !== generation) return false;
    pendingState.value = false;
    return fail(reason);
  }
}

function onPopoverOpen(value: boolean): void {
  if (value) setOpen(true);
  else cancel(null, "dismiss");
}

popconfirmContext.provide({
  cancel,
  cancelElement,
  cancelId,
  confirmId,
  confirm,
  confirmElement,
  descriptionId,
  error: shallowReadonly(errorState),
  open: isOpen,
  pending,
  state,
  titleId,
});

type PopconfirmRootSetupExpose = Omit<
  PopconfirmRootExpose,
  "descriptionId" | "error" | "id" | "open" | "pending" | "state" | "titleId"
> & {
  readonly descriptionId: ComputedRef<string>;
  readonly error: Readonly<ShallowRef<unknown>>;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly pending: ComputedRef<boolean>;
  readonly state: ComputedRef<PopconfirmState>;
  readonly titleId: ComputedRef<string>;
};

const exposed = {
  cancel: (nativeEvent = null) => cancel(nativeEvent, "programmatic"),
  confirm,
  descriptionId,
  error: shallowReadonly(errorState),
  id: baseId,
  open: isOpen,
  pending,
  setOpen,
  state,
  titleId,
} satisfies PopconfirmRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <PopoverRoot
    :id="baseId"
    :open="isOpen"
    :disabled
    data-vize-ui="popconfirm-root"
    part="root"
    :data-popconfirm-state="state"
    @update:open="onPopoverOpen"
  >
    <slot v-bind="slotState" />
  </PopoverRoot>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
