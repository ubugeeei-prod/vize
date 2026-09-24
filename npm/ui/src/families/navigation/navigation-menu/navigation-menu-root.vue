<script setup lang="ts">
import { computed, onScopeDispose, shallowReactive, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { navigationMenuContext } from "./navigation-menu-context.ts";
import type { NavigationMenuContextValue } from "./navigation-menu-context.ts";
import { getNavigationMenuIdSegment } from "./navigation-menu-id.ts";
import type {
  NavigationMenuChangeReason,
  NavigationMenuDirection,
  NavigationMenuMotion,
  NavigationMenuOrientation,
  NavigationMenuRootExpose,
  NavigationMenuSlotState,
  NavigationMenuValue,
} from "./navigation-menu-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = null,
  delayDuration = 200,
  skipDelayDuration = 300,
  closeDelay = 150,
  orientation = "horizontal",
  dir = "ltr",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open item value (`v-model`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: NavigationMenuValue;

  /**
   * Initially open item value for uncontrolled use.
   *
   * @default null
   */
  readonly defaultValue?: NavigationMenuValue;

  /**
   * Hover time in milliseconds before a pointer opens a flyout.
   *
   * @default 200
   */
  readonly delayDuration?: number;

  /**
   * Window in milliseconds after closing during which hovering another trigger opens instantly.
   *
   * @default 300
   */
  readonly skipDelayDuration?: number;

  /**
   * Grace time in milliseconds before a pointer leaving the trigger or flyout closes it.
   *
   * @default 150
   */
  readonly closeDelay?: number;

  /**
   * Layout axis of the top-level list, used for arrow keys and the open key.
   *
   * @default "horizontal"
   */
  readonly orientation?: NavigationMenuOrientation;

  /**
   * Reading direction for horizontal arrow keys and motion attributes.
   *
   * @default "ltr"
   */
  readonly dir?: NavigationMenuDirection;

  /**
   * Accessible name of the navigation landmark.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the landmark.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the next open item value whenever a flyout opens or closes. */
  "update:modelValue": [value: NavigationMenuValue];

  /** Fired after a distinct open-value change with the input that caused it. */
  change: [
    value: NavigationMenuValue,
    previous: NavigationMenuValue,
    reason: NavigationMenuChangeReason,
  ];
}>();

defineSlots<{
  /** List, item, and viewport children. Receives the open value and axis. */
  default(props: NavigationMenuSlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "navigation-menu" });
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const registry = createCollectionRegistry<string, string>();
const triggers = shallowReactive(new Map<string, Readonly<ShallowRef<HTMLButtonElement | null>>>());
const contents = shallowReactive(new Map<string, Readonly<ShallowRef<HTMLDivElement | null>>>());
const valueState = useControllableState<NavigationMenuValue>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const openValue = computed(() => valueState.value.value);
const previousValue = shallowRef<NavigationMenuValue>(null);
const slotState = computed<NavigationMenuSlotState>(() => ({
  open: openValue.value !== null,
  orientation: orientationState.value,
  value: openValue.value,
}));

let openTimer: ReturnType<typeof setTimeout> | null = null;
let closeTimer: ReturnType<typeof setTimeout> | null = null;
let skipDelayUntil = 0;

function clearOpenTimer(): void {
  if (openTimer !== null) clearTimeout(openTimer);
  openTimer = null;
}

function clearCloseTimer(): void {
  if (closeTimer !== null) clearTimeout(closeTimer);
  closeTimer = null;
}

function currentValue(): NavigationMenuValue {
  return openValue.value;
}

function setOpen(next: NavigationMenuValue, reason: NavigationMenuChangeReason): boolean {
  clearOpenTimer();
  clearCloseTimer();
  const previous = currentValue();
  if (next === previous) return false;
  if (next === null) skipDelayUntil = Date.now() + skipDelayDuration;
  previousValue.value = previous;
  valueState.set(next);
  emit("update:modelValue", next);
  emit("change", next, previous, reason);
  return true;
}

function toggle(value: string): void {
  setOpen(currentValue() === value ? null : value, "toggle");
}

function onTriggerEnter(value: string): void {
  clearCloseTimer();
  clearOpenTimer();
  if (currentValue() === value) return;
  if (currentValue() !== null || Date.now() < skipDelayUntil) {
    setOpen(value, "pointer");
    return;
  }
  openTimer = setTimeout(() => setOpen(value, "pointer"), delayDuration);
}

function onPointerLeave(): void {
  clearOpenTimer();
  if (currentValue() === null) return;
  clearCloseTimer();
  closeTimer = setTimeout(() => setOpen(null, "pointer"), closeDelay);
}

function getTriggerElement(value: NavigationMenuValue): HTMLButtonElement | null {
  return value === null ? null : (triggers.get(value)?.value ?? null);
}

function getContentElement(value: NavigationMenuValue): HTMLDivElement | null {
  return value === null ? null : (contents.get(value)?.value ?? null);
}

function focusable(container: Element): HTMLElement[] {
  const selector =
    "a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";
  return [...container.querySelectorAll<HTMLElement>(selector)];
}

function focusContent(value: string): void {
  const content = getContentElement(value);
  if (content !== null) focusable(content)[0]?.focus();
}

function indexOf(value: NavigationMenuValue): number {
  return value === null ? -1 : registry.items.value.findIndex((item) => item.key === value);
}

function readPreviousValue(): NavigationMenuValue {
  return previousValue.value;
}

function rootElement(): HTMLElement | null {
  return element.value;
}

function getMotion(value: string): NavigationMenuMotion | null {
  const open = currentValue();
  const previous = readPreviousValue();
  if (open === null || previous === null) return null;
  const rtl = dirState.value === "rtl";
  if (value === open) {
    const after = indexOf(open) > indexOf(previous);
    return after !== rtl ? "from-end" : "from-start";
  }
  if (value === previous) {
    const after = indexOf(open) > indexOf(previous);
    return after !== rtl ? "to-start" : "to-end";
  }
  return null;
}

function entries(): HTMLElement[] {
  const root = rootElement();
  if (root === null) return [];
  return [...root.querySelectorAll<HTMLElement>("[data-navigation-menu-entry]")].filter(
    (entry) => entry.closest('[data-vize-ui="navigation-menu"]') === root,
  );
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented) return;
  if (event.key === "Escape") {
    const open = currentValue();
    if (open === null) return;
    const trigger = getTriggerElement(open);
    setOpen(null, "dismiss");
    trigger?.focus();
    event.preventDefault();
    return;
  }
  const target = event.target;
  if (!(target instanceof HTMLElement) || !target.hasAttribute("data-navigation-menu-entry")) {
    return;
  }
  const list = entries();
  const position = list.indexOf(target);
  if (position < 0) return;
  const horizontal = orientationState.value === "horizontal";
  const rtl = dirState.value === "rtl";
  const nextKey = horizontal ? (rtl ? "ArrowLeft" : "ArrowRight") : "ArrowDown";
  const previousKey = horizontal ? (rtl ? "ArrowRight" : "ArrowLeft") : "ArrowUp";
  let next: HTMLElement | undefined;
  if (event.key === nextKey) next = list[(position + 1) % list.length];
  else if (event.key === previousKey) next = list[(position - 1 + list.length) % list.length];
  else if (event.key === "Home") next = list[0];
  else if (event.key === "End") next = list.at(-1);
  if (next === undefined) return;
  event.preventDefault();
  next.focus();
}

function onFocusout(event: FocusEvent): void {
  const root = rootElement();
  const next = event.relatedTarget;
  if (root === null || currentValue() === null) return;
  if (next instanceof Node && root.contains(next)) return;
  setOpen(null, "dismiss");
}

let stopOutside: (() => void) | null = null;
watch(openValue, (next) => {
  stopOutside?.();
  stopOutside = null;
  const root = rootElement();
  if (next === null || root === null) return;
  const doc = root.ownerDocument;
  const onPointerdown = (event: PointerEvent) => {
    if (event.target instanceof Node && root.contains(event.target)) return;
    setOpen(null, "dismiss");
  };
  doc.addEventListener("pointerdown", onPointerdown, true);
  stopOutside = () => doc.removeEventListener("pointerdown", onPointerdown, true);
});

onScopeDispose(() => {
  clearOpenTimer();
  clearCloseTimer();
  stopOutside?.();
});

const rootProps = computed<{
  readonly onFocusout: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ onFocusout, onKeydown }));

navigationMenuContext.provide({
  dir: dirState,
  focusContent,
  getContentElement,
  getContentId: (value) =>
    deriveDeterministicId(baseId.value, `content-${getNavigationMenuIdSegment(value)}`),
  getMotion,
  getTriggerElement,
  getTriggerId: (value) =>
    deriveDeterministicId(baseId.value, `trigger-${getNavigationMenuIdSegment(value)}`),
  onContentEnter: () => {
    clearCloseTimer();
  },
  onPointerLeave,
  onTriggerEnter,
  orientation: orientationState,
  registerContent: (value, target) => {
    contents.set(value, target);
    return () => {
      if (contents.get(value) === target) contents.delete(value);
    };
  },
  registerItem: (input) =>
    registry.register({ key: input.value, value: input.value, element: input.element }),
  registerTrigger: (value, target) => {
    triggers.set(value, target);
    return () => {
      if (triggers.get(value) === target) triggers.delete(value);
    };
  },
  setOpen,
  toggle,
  value: openValue,
} satisfies NavigationMenuContextValue);

type NavigationMenuRootSetupExpose = Omit<NavigationMenuRootExpose, "element" | "value"> & {
  readonly element: typeof element;
  readonly value: ComputedRef<NavigationMenuValue>;
};

const exposed = {
  close: () => setOpen(null, "programmatic"),
  element,
  open: (value: string) => setOpen(value, "programmatic"),
  value: openValue,
} satisfies NavigationMenuRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <nav
    v-bind="rootProps"
    :id="baseId"
    ref="element"
    :dir="dirState"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="navigation-menu"
    part="root"
    :data-state="openValue === null ? 'closed' : 'open'"
    :data-orientation="orientationState"
    :data-value="openValue ?? undefined"
  >
    <slot v-bind="slotState" />
  </nav>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
