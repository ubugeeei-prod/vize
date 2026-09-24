<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { PopoverTrigger } from "../popover/popover.ts";
import type { PopoverTriggerExpose } from "../popover/popover.ts";
import { popconfirmContext } from "./popconfirm-context.ts";
import type { PopconfirmButtonExpose, PopconfirmSlotState } from "./popconfirm-types.ts";

const { disabled = false, ariaLabel = undefined } = defineProps<{
  /**
   * Remove the trigger from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Trigger contents. Receives the confirmation state. */
  default?(props: PopconfirmSlotState): unknown;
}>();

const context = popconfirmContext.use();
const trigger = useTemplateRef<PopoverTriggerExpose>("trigger");
const element = computed(() => trigger.value?.element ?? null);
const slotState = computed<PopconfirmSlotState>(() => ({
  error: context.error.value,
  open: context.open.value,
  pending: context.pending.value,
  state: context.state.value,
}));

type PopconfirmTriggerSetupExpose = Omit<PopconfirmButtonExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus: (options?: FocusOptions) => trigger.value?.focus(options),
} satisfies PopconfirmTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <PopoverTrigger
    ref="trigger"
    :disabled
    :aria-label
    data-vize-ui="popconfirm-trigger"
    part="trigger"
    :data-popconfirm-state="context.state.value"
  >
    <slot v-bind="slotState" />
  </PopoverTrigger>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
