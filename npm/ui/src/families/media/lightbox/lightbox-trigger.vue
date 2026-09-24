<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { dialogContext } from "../../overlays/dialog/dialog-context.ts";
import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxButtonExpose, LightboxIndexSlotState } from "./lightbox-types.ts";

const { index, ariaLabel = undefined } = defineProps<{
  /** Zero-based item index opened by this trigger. @default required */
  readonly index: number;

  /**
   * Accessible name for thumbnail-only triggers without an `alt` text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before opening. Call `preventDefault()` to keep the viewer closed. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger content, typically a thumbnail image. */
  default(props: LightboxIndexSlotState): unknown;
}>();

const context = lightboxContext.use();
const dialog = dialogContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed<boolean>(() => index < 0 || index >= context.count.value);
const current = computed<boolean>(() => context.index.value === index);
const slotState = computed<LightboxIndexSlotState>(() => ({ current: current.value, index }));

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (event.defaultPrevented) return;
  // Focus returns to the trigger that opened the viewer.
  dialog.triggerElement.value = element.value;
  context.openAt(index);
}

type LightboxTriggerSetupExpose = Omit<LightboxButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies LightboxTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    aria-haspopup="dialog"
    :aria-expanded="context.open.value && current ? 'true' : 'false'"
    :aria-controls="dialog.contentId.value"
    data-vize-ui="lightbox-trigger"
    part="trigger"
    :data-index="index"
    :data-current="current ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
