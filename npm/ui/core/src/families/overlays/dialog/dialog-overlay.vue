<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { dialogContext } from "./dialog-context.ts";
import type { DialogOverlayExpose, DialogSlotState, DialogState } from "./dialog-types.ts";

const { forceMount = false } = defineProps<{
  /**
   * Keep the overlay mounted while the dialog is closed.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Optional overlay contents. Receives the current Dialog state. */
  default(props: DialogSlotState): unknown;
}>();

const context = dialogContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const present = computed(() => context.open.value || forceMount);
const slotState = computed<DialogSlotState>(() => ({
  modal: context.modal.value,
  open: context.open.value,
  state: context.state.value,
}));

watch(
  element,
  (next, previous) => {
    if (previous && context.overlayElement.value === previous) context.overlayElement.value = null;
    if (next) context.overlayElement.value = next;
  },
  { flush: "post" },
);

onUnmounted(() => {
  if (context.overlayElement.value === element.value) context.overlayElement.value = null;
});

type DialogOverlaySetupExpose = Omit<
  DialogOverlayExpose,
  "element" | "modal" | "open" | "state"
> & {
  readonly element: typeof element;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<DialogState>;
};

const exposed = {
  element,
  modal: context.modal,
  open: context.open,
  state: context.state,
} satisfies DialogOverlaySetupExpose;

defineExpose(exposed);
</script>

<template>
  <div data-vize-ui="dialog-overlay-host" part="overlay-host" :data-state="context.state.value">
    <div
      v-if="present"
      ref="element"
      aria-hidden="true"
      data-vize-ui="dialog-overlay"
      part="overlay"
      :hidden="context.open.value ? undefined : true"
      :data-state="context.state.value"
    >
      <slot v-bind="slotState" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
