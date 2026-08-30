<script setup lang="ts">
import { computed, onUnmounted, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { deriveDeterministicId, useDeterministicId } from "../../../deterministic-id.ts";
import { tooltipContext } from "./tooltip-context.ts";
import type { TooltipRootExpose, TooltipSlotState, TooltipState } from "./tooltip-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
  delayDuration = 700,
  skipDelayDuration = 300,
} = defineProps<{
  /**
   * Consumer-owned Tooltip base id. `null` and `undefined` select a deterministic fallback.
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
   * Disable trigger-driven opening and request closure when already open.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Milliseconds before hover or focus opens the tooltip.
   *
   * @default 700
   */
  readonly delayDuration?: number;

  /**
   * Milliseconds after closing where a new hover or focus opens without delay.
   *
   * @default 300
   */
  readonly skipDelayDuration?: number;
}>();

const emit = defineEmits<{
  /** Fired when the Tooltip requests a controlled open value. */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Tooltip children. Receives the current open and disabled state. */
  default(props: TooltipSlotState): unknown;
}>();

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const disabledState = computed(() => disabled);
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<TooltipState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "tooltip" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const delayState = computed(() => normalizeDelay(delayDuration));
const skipDelayState = computed(() => normalizeDelay(skipDelayDuration));
const slotState = computed<TooltipSlotState>(() => ({
  disabled: disabledState.value,
  open: isOpen.value,
  state: state.value,
}));
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const contentElement = shallowRef<HTMLDivElement | null>(null);

let openTimer: ReturnType<typeof setTimeout> | null = null;
let skipDelayUntil = 0;

function normalizeDelay(value: number): number {
  return Number.isFinite(value) && value > 0 ? value : 0;
}

function cancelOpen(): boolean {
  if (openTimer == null) return false;
  clearTimeout(openTimer);
  openTimer = null;
  return true;
}

function rememberSkipDelay(): void {
  skipDelayUntil = skipDelayState.value > 0 ? Date.now() + skipDelayState.value : 0;
}

function commitOpen(
  value: boolean,
  nativeEvent: Event | null = null,
  previous = isOpen.value,
): boolean {
  const next = value && !disabledState.value;
  const changed = openState.set(next);
  if (changed || previous !== next) {
    emit("update:open", next);
    emit("open-change", next, previous, nativeEvent);
  }
  return changed || previous !== next;
}

function openTooltip(nativeEvent: Event | null = null): boolean {
  cancelOpen();
  return commitOpen(true, nativeEvent);
}

function closeWithPrevious(nativeEvent: Event | null, previous: boolean): boolean {
  const cancelled = cancelOpen();
  if (previous || openState.value.value) rememberSkipDelay();
  return commitOpen(false, nativeEvent, previous) || cancelled;
}

function close(nativeEvent: Event | null = null): boolean {
  return closeWithPrevious(nativeEvent, isOpen.value);
}

function scheduleOpen(nativeEvent: Event | null = null): boolean {
  if (disabledState.value || isOpen.value) return false;
  if (delayState.value === 0 || Date.now() <= skipDelayUntil) return openTooltip(nativeEvent);
  if (openTimer != null) return false;

  openTimer = setTimeout(() => {
    openTimer = null;
    commitOpen(true, nativeEvent);
  }, delayState.value);
  return true;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  cancelOpen();
  if (!value && isOpen.value) rememberSkipDelay();
  return commitOpen(value, nativeEvent);
}

watch(disabledState, (next, previousDisabled) => {
  if (next && openState.value.value) {
    closeWithPrevious(null, !previousDisabled && openState.value.value);
  }
});

onUnmounted(cancelOpen);

const context = tooltipContext.provide({
  id: baseId,
  triggerId,
  contentId,
  open: isOpen,
  disabled: disabledState,
  state,
  delayDuration: delayState,
  skipDelayDuration: skipDelayState,
  triggerElement,
  contentElement,
  setOpen,
  openTooltip,
  close,
  scheduleOpen,
  cancelOpen,
});

type TooltipRootSetupExpose = Omit<
  TooltipRootExpose,
  | "contentId"
  | "delayDuration"
  | "disabled"
  | "id"
  | "open"
  | "skipDelayDuration"
  | "state"
  | "triggerId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly delayDuration: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly skipDelayDuration: ComputedRef<number>;
  readonly state: ComputedRef<TooltipState>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  cancelOpen,
  close,
  contentId,
  delayDuration: delayState,
  disabled: disabledState,
  id: baseId,
  open: isOpen,
  openTooltip,
  scheduleOpen,
  setOpen,
  skipDelayDuration: skipDelayState,
  state,
  triggerId,
} satisfies TooltipRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="tooltip-root"
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
