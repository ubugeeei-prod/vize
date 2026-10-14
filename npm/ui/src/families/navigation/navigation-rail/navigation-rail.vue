<script setup lang="ts" generic="Value extends string">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { navigationRailContext } from "./navigation-rail-context.ts";
import type { NavigationRailExpose, NavigationRailSlotState } from "./navigation-rail-types.ts";

const {
  destinations,
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  expanded = undefined,
  defaultExpanded = false,
  ariaLabel = "Primary",
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Destination values in order; their literal union types `v-model` and `select`.
   *
   * @default required
   */
  readonly destinations: readonly Value[];

  /**
   * Consumer-owned id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled active destination. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: Value;

  /**
   * Initial uncontrolled destination.
   *
   * @default undefined
   */
  readonly defaultValue?: Value;

  /**
   * Controlled expanded state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly expanded?: boolean;

  /**
   * Initial expanded state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultExpanded?: boolean;

  /**
   * Accessible name of the navigation landmark.
   *
   * @default "Primary"
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the navigation landmark (overrides `ariaLabel`).
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when an item requests to become the active destination. */
  "update:modelValue": [value: Value];

  /** Fired after an item is activated, with its value and the native click. */
  select: [value: Value, nativeEvent: MouseEvent];

  /** Fired with the requested expanded value (supports `v-model:expanded`). */
  "update:expanded": [value: boolean];
}>();

defineSlots<{
  /** Toggle and NavigationRailItem destinations with the active value and expanded state. */
  default(props: NavigationRailSlotState<Value>): unknown;
}>();

const railId = useDeterministicId({ id: () => id, hint: "navigation-rail" });
const state = useControllableState<Value | undefined>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => {
    if (value !== undefined) emit("update:modelValue", value);
  },
});
const expandedState = useControllableState<boolean>({
  value: () => expanded,
  defaultValue: () => defaultExpanded,
  onChange: (value) => emit("update:expanded", value),
});
const isExpanded = computed(() => expandedState.value.value);

function setExpanded(value: boolean): void {
  expandedState.set(value);
}

function toggle(): void {
  expandedState.set(!expandedState.value.value);
}

navigationRailContext.provide({
  id: railId,
  active: computed(() => state.value.value),
  expanded: isExpanded,
  select: (value: string, event: MouseEvent) => {
    const typed = destinations.find((destination) => destination === value);
    if (typed === undefined) return;
    state.set(typed);
    emit("select", typed, event);
  },
  setExpanded,
});

defineExpose({
  expanded: isExpanded,
  id: railId,
  setExpanded,
  toggle,
} satisfies Omit<NavigationRailExpose, "expanded" | "id"> & {
  readonly expanded: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
});
</script>

<template>
  <nav
    :id="railId"
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby
    part="root"
    data-vize-ui="navigation-rail"
    :data-count="destinations.length"
    :data-expanded="isExpanded ? 'true' : 'false'"
  >
    <slot :active="state.value.value" :expanded="isExpanded" />
  </nav>
</template>

<style scoped>
/* Headless by design. Pin with position: sticky or fixed and pad with SafeArea. */
</style>
