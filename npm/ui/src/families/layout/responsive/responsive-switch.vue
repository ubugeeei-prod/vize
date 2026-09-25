<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { defaultBreakpoints, useBreakpoint } from "./breakpoint.ts";
import type { BreakpointMap, ResponsiveSwitchSlotState } from "./responsive-types.ts";

const {
  as = "div",
  breakpoints = defaultBreakpoints,
  ssrWidth = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Named minimum widths in CSS pixels.
   *
   * @default defaultBreakpoints
   */
  readonly breakpoints?: BreakpointMap;

  /**
   * Viewport width assumed during server rendering and hydration.
   *
   * @default undefined
   */
  readonly ssrWidth?: number;
}>();

const slots = defineSlots<{
  /** Rendered when no breakpoint-named slot matches. Receives the active breakpoint. */
  default?(props: ResponsiveSwitchSlotState): unknown;
  /** Rendered below every breakpoint (before any breakpoint is reached). */
  base?(props: ResponsiveSwitchSlotState): unknown;
  /** Breakpoint-named slots (`sm`, `md`, ...): the largest reached one with content wins. */
  [breakpoint: string]: ((props: ResponsiveSwitchSlotState) => unknown) | undefined;
}>();

const element = useTemplateRef<Element>("element");
const breakpoint = useBreakpoint(() => breakpoints, { ssrWidth: () => ssrWidth });
const slotState = computed<ResponsiveSwitchSlotState>(() => ({
  active: breakpoint.active.value,
  width: breakpoint.width.value,
}));

/** Largest reached breakpoint that has a slot, falling back to `base` then `default`. */
const slotName = computed(() => {
  const width = breakpoint.width.value;
  const reached = Object.entries(breakpoints)
    .filter(([, minimum]) => width !== null && width >= minimum)
    .sort(([, left], [, right]) => right - left)
    .map(([name]) => name);
  const named = reached.find((name) => slots[name] !== undefined);
  if (named !== undefined) return named;
  if (width !== null && slots["base"] !== undefined) return "base";
  return "default";
});

defineExpose({ element, active: breakpoint.active, width: breakpoint.width });
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="responsive-switch"
    :data-breakpoint="breakpoint.active.value ?? 'base'"
    :data-slot="slotName"
  >
    <slot :name="slotName" v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Breakpoint slots only choose which content renders. */
</style>
