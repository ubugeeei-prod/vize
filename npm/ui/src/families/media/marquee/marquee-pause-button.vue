<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { marqueeContext } from "./marquee-context.ts";
import type { MarqueePauseButtonExpose, MarqueeSlotState } from "./marquee-types.ts";

const emit = defineEmits<{
  /** Fired before toggling. Call `preventDefault()` to keep the play intent. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content, e.g. an icon. The accessible name comes from the messages. */
  default?(props: MarqueeSlotState & { readonly label: string }): unknown;
}>();

const context = marqueeContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const label = computed<string>(() =>
  context.effectivePlaying.value ? context.messages.value.pause : context.messages.value.play,
);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.toggle();
}

const exposed = { element } satisfies Omit<MarqueePauseButtonExpose, "element"> & {
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :aria-label="label"
    :aria-controls="context.id.value"
    data-vize-ui="marquee-pause-button"
    part="pause-button"
    :data-state="context.effectivePlaying.value ? 'playing' : 'paused'"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" :label />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
