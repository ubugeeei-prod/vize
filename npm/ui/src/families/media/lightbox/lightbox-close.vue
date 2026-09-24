<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxButtonExpose, LightboxPartSlotState } from "./lightbox-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to the `close` message.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before closing. Call `preventDefault()` to keep the viewer open. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content, e.g. an icon. */
  default?(props: LightboxPartSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const label = computed<string>(() => ariaLabel ?? context.messages.value.close);
const disabled = computed<boolean>(() => false);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.close();
}

type LightboxCloseSetupExpose = Omit<LightboxButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies LightboxCloseSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :aria-label="label"
    data-vize-ui="lightbox-close"
    part="close"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
