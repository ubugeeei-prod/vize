<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { popconfirmContext } from "./popconfirm-context.ts";
import type { PopconfirmButtonExpose, PopconfirmSlotState } from "./popconfirm-types.ts";

const { disabled = false } = defineProps<{
  /**
   * Disable this action in addition to the pending state.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired before the action runs. Call `preventDefault()` to skip it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Close without confirming. Disabled while a confirmation is pending. */
  default?(props: PopconfirmSlotState): unknown;
}>();

const context = popconfirmContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const blocked = computed(() => disabled || context.pending.value);
const slotState = computed<PopconfirmSlotState>(() => ({
  error: context.error.value,
  open: context.open.value,
  pending: context.pending.value,
  state: context.state.value,
}));

onMounted(() => {
  context.cancelElement.value = element.value;
});

onUnmounted(() => {
  if (context.cancelElement.value === element.value) context.cancelElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (blocked.value) return;
  emit("click", event);
  if (event.defaultPrevented) return;
  context.cancel(event, "cancel-button");
}

type PopconfirmCancelSetupExpose = Omit<PopconfirmButtonExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus: (options?: FocusOptions) => element.value?.focus(options),
} satisfies PopconfirmCancelSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.cancelId.value"
    ref="element"
    type="button"
    :disabled="blocked"
    data-vize-ui="popconfirm-cancel"
    part="cancel"
    :data-state="context.state.value"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
