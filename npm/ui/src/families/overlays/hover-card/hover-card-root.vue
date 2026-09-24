<script setup lang="ts">
import { computed, onScopeDispose, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { hoverCardContext } from "./hover-card-context.ts";
import type {
  HoverCardOpenReason,
  HoverCardRootExpose,
  HoverCardSlotState,
  HoverCardState,
  HoverCardTouchBehavior,
} from "./hover-card-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
  openDelay = 700,
  closeDelay = 300,
  touchBehavior = "ignore",
  longPressDelay = 500,
} = defineProps<{
  /**
   * Consumer-owned HoverCard base id. `null` and `undefined` select a deterministic fallback.
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
   * Disable pointer, focus, and touch opening and close an open card.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Milliseconds the pointer or focus must rest on the trigger before opening.
   *
   * @default 700
   */
  readonly openDelay?: number;

  /**
   * Milliseconds after the pointer or focus leaves before closing. Moving
   * into the card or through the pointer-grace corridor cancels the close.
   *
   * @default 300
   */
  readonly closeDelay?: number;

  /**
   * Touch handling on the trigger. Hover cards are supplementary, so touch is
   * ignored by default and taps keep native link behavior.
   *
   * @default "ignore"
   */
  readonly touchBehavior?: HoverCardTouchBehavior;

  /**
   * Milliseconds a touch must be held when `touchBehavior` is `"long-press"`.
   *
   * @default 500
   */
  readonly longPressDelay?: number;
}>();

const emit = defineEmits<{
  /** Fired when the HoverCard requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound HoverCard children. Receives the current open state. */
  default(props: HoverCardSlotState): unknown;
}>();

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const disabledState = computed(() => disabled);
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<HoverCardState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "hover-card" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const openDelayState = computed(() => normalizeDelay(openDelay));
const closeDelayState = computed(() => normalizeDelay(closeDelay));
const touchBehaviorState = computed(() => touchBehavior);
const longPressDelayState = computed(() => normalizeDelay(longPressDelay));
const openReason = shallowRef<HoverCardOpenReason | null>(null);
const reason = computed(() => (isOpen.value ? openReason.value : null));
const triggerElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLDivElement | null>(null);
const slotState = computed<HoverCardSlotState>(() => ({
  disabled: disabledState.value,
  open: isOpen.value,
  reason: reason.value,
  state: state.value,
}));

let openTimer: ReturnType<typeof setTimeout> | null = null;
let closeTimer: ReturnType<typeof setTimeout> | null = null;

function readOpen(): boolean {
  return isOpen.value;
}

function normalizeDelay(value: number): number {
  return Number.isFinite(value) && value > 0 ? value : 0;
}

function cancelOpenTimer(): boolean {
  if (openTimer === null) return false;
  clearTimeout(openTimer);
  openTimer = null;
  return true;
}

function cancelCloseTimer(): boolean {
  if (closeTimer === null) return false;
  clearTimeout(closeTimer);
  closeTimer = null;
  return true;
}

function cancelPending(): boolean {
  const cancelledOpen = cancelOpenTimer();
  return cancelCloseTimer() || cancelledOpen;
}

function commitOpen(
  value: boolean,
  nativeEvent: Event | null,
  nextReason: HoverCardOpenReason,
): boolean {
  const next = value && !disabledState.value;
  const previous = readOpen();
  if (next) openReason.value = nextReason;
  const changed = openState.set(next);
  if (changed) {
    emit("update:open", next);
    emit("open-change", next, previous, nativeEvent);
  }
  return changed;
}

function setOpen(
  value: boolean,
  nativeEvent: Event | null = null,
  nextReason: HoverCardOpenReason = "programmatic",
): boolean {
  cancelPending();
  return commitOpen(value, nativeEvent, nextReason);
}

function scheduleOpen(
  nativeEvent: Event | null = null,
  nextReason: HoverCardOpenReason = "hover",
): boolean {
  cancelCloseTimer();
  if (disabledState.value || isOpen.value) return false;
  if (openDelayState.value === 0) return commitOpen(true, nativeEvent, nextReason);
  if (openTimer !== null) return false;
  openTimer = setTimeout(() => {
    openTimer = null;
    commitOpen(true, nativeEvent, nextReason);
  }, openDelayState.value);
  return true;
}

function scheduleClose(nativeEvent: Event | null = null): boolean {
  const cancelledOpen = cancelOpenTimer();
  if (!isOpen.value) return cancelledOpen;
  if (closeDelayState.value === 0) return commitOpen(false, nativeEvent, "programmatic");
  if (closeTimer !== null) return false;
  closeTimer = setTimeout(() => {
    closeTimer = null;
    commitOpen(false, nativeEvent, "programmatic");
  }, closeDelayState.value);
  return true;
}

watch(disabledState, (next) => {
  if (!next) return;
  cancelPending();
  if (openState.value.value) {
    const changed = openState.set(false);
    if (changed) {
      emit("update:open", false);
      emit("open-change", false, true, null);
    }
  }
});

onScopeDispose(cancelPending);

hoverCardContext.provide({
  cancelPending,
  contentElement,
  contentId,
  disabled: disabledState,
  id: baseId,
  longPressDelay: longPressDelayState,
  open: isOpen,
  reason,
  scheduleClose,
  scheduleOpen,
  setOpen,
  state,
  touchBehavior: touchBehaviorState,
  triggerElement,
  triggerId,
});

type HoverCardRootSetupExpose = Omit<
  HoverCardRootExpose,
  | "closeDelay"
  | "contentId"
  | "disabled"
  | "id"
  | "open"
  | "openDelay"
  | "reason"
  | "state"
  | "triggerId"
> & {
  readonly closeDelay: ComputedRef<number>;
  readonly contentId: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly openDelay: ComputedRef<number>;
  readonly reason: Readonly<ShallowRef<HoverCardOpenReason | null>>;
  readonly state: ComputedRef<HoverCardState>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  cancelPending,
  closeDelay: closeDelayState,
  contentId,
  disabled: disabledState,
  id: baseId,
  open: isOpen,
  openDelay: openDelayState,
  reason,
  scheduleClose: (nativeEvent = null) => scheduleClose(nativeEvent),
  scheduleOpen: (nativeEvent = null) => scheduleOpen(nativeEvent, "programmatic"),
  setOpen: (value, nativeEvent = null) => setOpen(value, nativeEvent),
  state,
  triggerId,
} satisfies HoverCardRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    :id="baseId"
    data-vize-ui="hover-card-root"
    part="root"
    :data-state="state"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
