<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { panZoomContext } from "./pan-zoom-context.ts";
import type { PanZoomButtonExpose, PanZoomSlotState } from "./pan-zoom-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name for icon-only content. Without slot content the localized
   * `messages.fit` label is rendered as text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before the action. Call `preventDefault()` to keep the transform. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content. Receives the current transform state. */
  default(props: PanZoomSlotState): unknown;
}>();

const context = panZoomContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => context.disabled.value);
const label = computed(() => context.messages.value.fit);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.fit("button");
}

type PanZoomButtonSetupExpose = Omit<PanZoomButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies PanZoomButtonSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    data-vize-ui="pan-zoom-fit"
    part="fit"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value">{{ label }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
