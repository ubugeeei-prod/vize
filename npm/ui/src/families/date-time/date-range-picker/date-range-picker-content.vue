<script setup lang="ts">
import PopoverContent from "../../overlays/popover/popover-content.vue";
import type { Placement } from "../../overlays/positioner/positioner.ts";
import { dateRangePickerContext } from "./date-range-picker-context.ts";

const {
  placement = "bottom-start",
  initialFocus = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /** Preferred placement before collision handling. @default "bottom-start" */
  readonly placement?: Placement;
  /** Initial focus target; defaults to the roving calendar day. @default undefined */
  readonly initialFocus?: () => HTMLElement | null | undefined;
  /** Accessible dialog name. @default undefined */
  readonly ariaLabel?: string;
  /** Ids that label the dialog. @default undefined */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Popover body, typically DateRangePickerCalendar. */
  default(props: Record<string, never>): unknown;
}>();

const context = dateRangePickerContext.use();

function resolveInitialFocus(): HTMLElement | null | undefined {
  return initialFocus?.() ?? context.focusTarget();
}
</script>

<template>
  <PopoverContent
    :placement
    :initial-focus="resolveInitialFocus"
    :aria-label
    :aria-labelledby
    data-picker-part="content"
  >
    <slot />
  </PopoverContent>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
