<script setup lang="ts">
import { computed, nextTick, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import type {
  SpeedDialChangeReason,
  SpeedDialDirection,
  SpeedDialRootExpose,
  SpeedDialSelectEvent,
  SpeedDialSlotState,
  SpeedDialState,
} from "./floating-action-button-types.ts";
import { speedDialContext } from "./speed-dial-context.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  direction = "up",
  disabled = false,
  openOnHover = false,
  closeOnSelect = true,
  loop = true,
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
   * Direction the actions fan out; also selects the arrow keys that move between actions.
   *
   * @default "up"
   */
  readonly direction?: SpeedDialDirection;

  /**
   * Disable opening and every action.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Open while a mouse or pen hovers the speed dial and close when it leaves.
   *
   * @default false
   */
  readonly openOnHover?: boolean;

  /**
   * Close and return focus to the trigger after an action is selected.
   *
   * @default true
   */
  readonly closeOnSelect?: boolean;

  /**
   * Whether arrow-key navigation wraps at the first and last action.
   *
   * @default true
   */
  readonly loop?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the speed dial requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request, with its cause. */
  "open-change": [value: boolean, reason: SpeedDialChangeReason, nativeEvent: Event | null];

  /** Fired when an action is selected. Call `preventDefault()` to keep the speed dial open. */
  select: [event: SpeedDialSelectEvent];
}>();

defineSlots<{
  /** Trigger and content. Receives the open state and direction. */
  default(props: SpeedDialSlotState): unknown;
}>();

const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const disabledState = computed(() => disabled);
const directionState = computed(() => direction);
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<SpeedDialState>(() => (isOpen.value ? "open" : "closed"));
const baseId = useDeterministicId({ id: () => id, hint: "speed-dial" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const contentElement = shallowRef<HTMLDivElement | null>(null);
const registry = createCollectionRegistry<string, string>({ disabledBehavior: "skip" });
const navigation = useCompositeNavigation<string, string>({
  registry,
  focusStrategy: "roving",
  orientation: () => (direction === "up" || direction === "down" ? "vertical" : "horizontal"),
  direction: () => (direction === "left" ? "rtl" : "ltr"),
  loop: () => loop,
  isDisabled: disabledState,
});
const slotState = computed<SpeedDialSlotState>(() => ({
  direction,
  disabled: disabledState.value,
  open: isOpen.value,
  state: state.value,
}));
let pendingReason: SpeedDialChangeReason = "programmatic";

function readOpen(): boolean {
  return isOpen.value;
}

function setOpenWithReason(
  value: boolean,
  reason: SpeedDialChangeReason,
  nativeEvent: Event | null,
): boolean {
  const next = value && !disabledState.value;
  if (next === readOpen()) return false;
  pendingReason = reason;
  openState.set(next);
  emit("update:open", next);
  emit("open-change", next, pendingReason, nativeEvent);
  return true;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  return setOpenWithReason(value, nativeEvent ? "pointer" : "programmatic", nativeEvent);
}

function openAndFocus(nativeEvent: Event | null = null): boolean {
  const changed = setOpenWithReason(true, nativeEvent ? "keyboard" : "programmatic", nativeEvent);
  void nextTick(() => {
    if (readOpen()) navigation.navigate("first", nativeEvent);
  });
  return changed;
}

function close(
  options: { readonly focusTrigger?: boolean } = {},
  nativeEvent: Event | null = null,
  reason: SpeedDialChangeReason = "programmatic",
): boolean {
  const changed = setOpenWithReason(false, reason, nativeEvent);
  if (options.focusTrigger) triggerElement.value?.focus();
  return changed;
}

function select(value: string, originalEvent: Event): SpeedDialSelectEvent {
  let prevented = false;
  const event: SpeedDialSelectEvent = {
    get defaultPrevented() {
      return prevented;
    },
    originalEvent,
    preventDefault: () => {
      prevented = true;
    },
    value,
  };
  emit("select", event);
  if (!prevented && closeOnSelect) close({ focusTrigger: true }, originalEvent, "action");
  return event;
}

function onPointerenter(event: PointerEvent): void {
  if (openOnHover && event.pointerType !== "touch") setOpenWithReason(true, "hover", event);
}

function onPointerleave(event: PointerEvent): void {
  if (openOnHover && event.pointerType !== "touch") setOpenWithReason(false, "hover", event);
}

speedDialContext.provide({
  close: (options, nativeEvent = null) =>
    close(options, nativeEvent, nativeEvent instanceof KeyboardEvent ? "escape" : "outside"),
  contentElement,
  contentId,
  direction: directionState,
  disabled: disabledState,
  id: baseId,
  navigation,
  open: isOpen,
  openAndFocus,
  registry,
  select,
  setOpen,
  state,
  triggerElement,
  triggerId,
});

type SpeedDialRootSetupExpose = Omit<
  SpeedDialRootExpose,
  "contentId" | "direction" | "disabled" | "id" | "open" | "state" | "triggerId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly direction: ComputedRef<SpeedDialDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<SpeedDialState>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  close: (options = {}) => close(options),
  contentId,
  direction: directionState,
  disabled: disabledState,
  id: baseId,
  open: isOpen,
  openAndFocus: (nativeEvent = null) => openAndFocus(nativeEvent),
  setOpen: (value, nativeEvent = null) => setOpen(value, nativeEvent),
  state,
  triggerId,
} satisfies SpeedDialRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="speed-dial-root"
    part="root"
    :data-state="state"
    :data-direction="direction"
    :data-disabled="disabled ? 'true' : undefined"
    @pointerenter="onPointerenter"
    @pointerleave="onPointerleave"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
