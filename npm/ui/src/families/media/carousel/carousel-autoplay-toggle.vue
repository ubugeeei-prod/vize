<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { carouselContext } from "./carousel-context.ts";
import type { CarouselButtonExpose, CarouselSlotState } from "./carousel-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name for icon-only controls. Prefer slot text that changes with
   * the state, e.g. "Stop slide rotation" / "Start slide rotation".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before toggling. Call `preventDefault()` to keep the rotation intent. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Control label. Receives the carousel state; `autoplay` tells which label to show. */
  default(props: CarouselSlotState): unknown;
}>();

const context = carouselContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => context.slideCount.value < 2);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.setPlaying(!context.playing.value);
}

type CarouselAutoplayToggleSetupExpose = Omit<CarouselButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies CarouselAutoplayToggleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    :aria-controls="context.viewportId.value"
    data-vize-ui="carousel-autoplay-toggle"
    part="autoplay-toggle"
    :data-state="context.autoplay.value"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
