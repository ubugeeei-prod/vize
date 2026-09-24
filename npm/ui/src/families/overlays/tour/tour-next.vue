<script setup lang="ts">
import { computed } from "vue";

import { tourContext } from "./tour-context.ts";
import type { TourControlSlotState } from "./tour-types.ts";

const { disabled = false, ariaLabel = undefined } = defineProps<{
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
}>();

const emit = defineEmits<{
  /** Fired before moving to the next step or completing on the last step. Call `preventDefault()` to keep the tour unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Control label. Receives position and availability state; defaults to the TourRoot `messages` label. */
  default(props: TourControlSlotState): unknown;
}>();

const context = tourContext.use();
const controlDisabled = computed(() => disabled || !context.open.value || context.pending.value);
const label = computed(() =>
  context.last.value ? context.messages.value.finish : context.messages.value.next,
);
const slotState = computed<TourControlSlotState>(() => ({
  disabled: controlDisabled.value,
  first: context.first.value,
  label: label.value,
  last: context.last.value,
  pending: context.pending.value,
  state: context.state.value,
}));

function onClick(event: MouseEvent): void {
  if (controlDisabled.value) return;
  emit("click", event);
  if (!event.defaultPrevented) context.next(event);
}
</script>

<template>
  <button
    type="button"
    :disabled="controlDisabled"
    :aria-label="ariaLabel"
    data-vize-ui="tour-next"
    part="next"
    :data-state="context.state.value"
    :data-disabled="controlDisabled ? 'true' : undefined"
    :data-last="context.last.value ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState">{{ label }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
