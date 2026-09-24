<script setup lang="ts" generic="Value extends string">
import { computed } from "vue";

import { bottomNavigationContext } from "./bottom-navigation-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";

const {
  destinations,
  modelValue = undefined,
  defaultValue = undefined,
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
}>();

defineSlots<{
  /** Renders BottomNavigationItem destinations with the active value. */
  default(props: { readonly active: Value | undefined }): unknown;
}>();

const state = useControllableState<Value | undefined>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => {
    if (value !== undefined) emit("update:modelValue", value);
  },
});

bottomNavigationContext.provide({
  active: computed(() => state.value.value),
  select: (value: string, event: MouseEvent) => {
    const typed = destinations.find((destination) => destination === value);
    if (typed === undefined) return;
    state.set(typed);
    emit("select", typed, event);
  },
});
</script>

<template>
  <nav
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby="ariaLabelledby"
    part="root"
    data-vize-ui="bottom-navigation"
    :data-count="destinations.length"
  >
    <slot :active="state.value.value" />
  </nav>
</template>

<style scoped>
/* Headless by design. Pin with position: fixed and pad with SafeArea. */
</style>
