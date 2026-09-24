<script setup lang="ts">
import { computed } from "vue";

import { calendarContext } from "./calendar-context.ts";
import type { CalendarNavigationSlotState, CalendarNavigationUnit } from "./calendar-types.ts";

const { unit = "month", ariaLabel = undefined } = defineProps<{
  /** Whether the control pages by month (or `numberOfMonths` when paged) or by year. @default "month" */
  readonly unit?: CalendarNavigationUnit;
  /** Accessible name; defaults to "Previous month" or "Previous year". @default undefined */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before navigation. Call `preventDefault()` to keep the view unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Control content. Receives the unit and whether navigation is possible. */
  default(props: CalendarNavigationSlotState): unknown;
}>();

const context = calendarContext.use();
const disabled = computed(() => !context.canNavigate(unit, -1));
const label = computed(() => ariaLabel ?? `Previous ${unit}`);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.navigate(unit, -1);
}
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    :aria-label="label"
    :aria-controls="context.id.value"
    data-vize-ui="calendar-prev"
    part="prev"
    :data-unit="unit"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot :unit="unit" :disabled="disabled">‹</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
