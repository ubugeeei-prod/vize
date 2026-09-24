<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useTypeahead } from "../../interaction/typeahead/typeahead.ts";
import { focusElement } from "../menu/menu-dom.ts";
import type { MenuDirection, MenuEntryFocus } from "../menu/menu-types.ts";
import { menubarContext } from "./menubar-context.ts";
import type { MenubarMenuRecord, MenubarMoveDirection } from "./menubar-context.ts";
import type { MenubarRootExpose, MenubarSlotState } from "./menubar-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = null,
  dir = "ltr",
  loop = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned menubar id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled value of the open menu (`v-model`); `null` closes every menu.
   * `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string | null;

  /**
   * Initially open menu for uncontrolled use.
   *
   * @default null
   */
  readonly defaultValue?: string | null;

  /**
   * Reading direction: flips Left/Right arrow meaning across triggers and menus.
   *
   * @default "ltr"
   */
  readonly dir?: MenuDirection;

  /**
   * Wrap Left/Right navigation from the last trigger to the first and back.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Accessible name for the menubar.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the menubar.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the requested open-menu value (supports `v-model`). */
  "update:modelValue": [value: string | null];
  /** Fired after a distinct open-menu change with the previous value and native event. */
  "value-change": [value: string | null, previous: string | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** MenubarMenu children. Receives the open-menu value. */
  default(props: MenubarSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const menubarId = useDeterministicId({ id: () => id, hint: "menubar" });
const valueState = useControllableState<string | null>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const dirState = computed(() => dir);
const loopState = computed(() => loop);
const registry = createCollectionRegistry<string, MenubarMenuRecord>({
  disabledBehavior: "focusable",
});

function focusTrigger(key: string): void {
  registry.setActiveKey(key);
  focusElement(registry.getItem(key)?.element);
}

const typeahead = useTypeahead({ registry, onMatch: (match) => focusTrigger(match.key) });

function setValue(next: string | null, event: Event | null): boolean {
  const previous = valueState.value.value;
  if (!valueState.set(next)) return false;
  emit("update:modelValue", next);
  emit("value-change", next, previous, event);
  return true;
}

function move(
  from: string,
  direction: MenubarMoveDirection,
  event: Event | null,
  entry: MenuEntryFocus = "first",
): boolean {
  const target =
    direction === "first" || direction === "last"
      ? registry.getNavigationKey(direction)
      : registry.getNavigationKey(direction, { fromKey: from, loop: loopState.value });
  if (target === null || target === from) return false;
  const wasOpen = valueState.value.value !== null;
  focusTrigger(target);
  if (wasOpen) switchTo(target, event, entry);
  return true;
}

function switchTo(key: string, event: Event | null, entry: MenuEntryFocus): boolean {
  const openValue = valueState.value.value;
  if (openValue === key) return false;
  const current = openValue === null ? undefined : registry.getItem(openValue)?.value.level;
  if (current) current.skipFocusReturn.value = true;
  return registry.getItem(key)?.value.level.setOpen(true, event, entry) ?? false;
}

menubarContext.provide({
  value: valueState.value,
  dir: dirState,
  loop: loopState,
  registry,
  typeahead,
  setValue,
  switchTo,
  move,
});

function focus(): void {
  const key = registry.activeKey.value ?? registry.getNavigationKey("first");
  if (key !== null) focusTrigger(key);
}

defineExpose({
  dir: dirState,
  element,
  focus,
  setValue: (value: string | null) => setValue(value, null),
  value: valueState.value,
} satisfies Omit<MenubarRootExpose, "dir" | "element" | "value"> & {
  readonly dir: typeof dirState;
  readonly element: typeof element;
  readonly value: typeof valueState.value;
});
</script>

<template>
  <div
    :id="menubarId"
    ref="element"
    role="menubar"
    aria-orientation="horizontal"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :dir
    data-vize-ui="menubar"
    part="root"
    :data-state="valueState.value.value === null ? 'closed' : 'open'"
  >
    <slot :dir :value="valueState.value.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
