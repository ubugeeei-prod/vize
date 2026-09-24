<script setup lang="ts">
import { computed } from "vue";

import { tourContext } from "./tour-context.ts";
import type { TourCloseReason, TourControlSlotState } from "./tour-types.ts";

const {
  disabled = false,
  ariaLabel = undefined,
  reason = "close",
} = defineProps<{
  /**
   * Disable this control.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when the visible label is not enough.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Dismissal reason reported by TourRoot. Use `skip` for "Skip tour" controls.
   *
   * @default "close"
   */
  readonly reason?: TourCloseReason;
}>();

const emit = defineEmits<{
  /** Fired before dismissing the tour. Call `preventDefault()` to keep the tour unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Control label. Receives position and availability state. */
  default(props: TourControlSlotState): unknown;
}>();

const context = tourContext.use();
const controlDisabled = computed(() => disabled || !context.open.value);
const slotState = computed<TourControlSlotState>(() => ({
  disabled: controlDisabled.value,
  first: context.first.value,
  last: context.last.value,
  state: context.state.value,
}));

function onClick(event: MouseEvent): void {
  if (controlDisabled.value) return;
  emit("click", event);
  if (!event.defaultPrevented) context.dismiss(reason, event);
}
</script>

<template>
  <button
    type="button"
    :disabled="controlDisabled"
    :aria-label="ariaLabel"
    data-vize-ui="tour-close"
    part="close"
    :data-state="context.state.value"
    :data-disabled="controlDisabled ? 'true' : undefined"
    :data-reason="reason"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
