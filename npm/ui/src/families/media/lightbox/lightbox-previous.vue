<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxButtonExpose, LightboxPartSlotState } from "./lightbox-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name. Defaults to the `previous` message.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before navigation. Call `preventDefault()` to keep the current item. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content, e.g. an icon. */
  default?(props: LightboxPartSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed<boolean>(() => !context.canGoPrevious.value);
const label = computed<string>(() => ariaLabel ?? context.messages.value.previous);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.step(-1, "previous");
}

type LightboxPreviousSetupExpose = Omit<LightboxButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies LightboxPreviousSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="label"
    data-vize-ui="lightbox-previous"
    part="previous"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
