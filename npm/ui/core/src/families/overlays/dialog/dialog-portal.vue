<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import { dialogContext } from "./dialog-context.ts";
import type { DialogPortalExpose, DialogSlotState, DialogState } from "./dialog-types.ts";
import Portal from "../../../portal.vue";

const {
  to = "body",
  disabled = false,
  defer = true,
  forceMount = false,
} = defineProps<{
  /**
   * CSS selector or element the dialog layer is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render in place instead of teleporting.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep content in place until the target exists, avoiding SSR mismatch.
   *
   * @default true
   */
  readonly defer?: boolean;

  /**
   * Keep the portal host mounted while the dialog is closed.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Portalled Dialog layer contents. */
  default(props: DialogSlotState): unknown;
}>();

const context = dialogContext.use();
const present = computed(() => context.open.value || forceMount);
const slotState = computed<DialogSlotState>(() => ({
  modal: context.modal.value,
  open: context.open.value,
  state: context.state.value,
}));

type DialogPortalSetupExpose = Omit<DialogPortalExpose, "modal" | "open" | "present" | "state"> & {
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly present: ComputedRef<boolean>;
  readonly state: ComputedRef<DialogState>;
};

const exposed = {
  modal: context.modal,
  open: context.open,
  present,
  state: context.state,
} satisfies DialogPortalSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="dialog-portal"
    part="portal"
    :hidden="present ? undefined : true"
    :data-state="context.state.value"
  >
    <Portal v-if="present" :to :disabled :defer>
      <slot v-bind="slotState" />
    </Portal>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
