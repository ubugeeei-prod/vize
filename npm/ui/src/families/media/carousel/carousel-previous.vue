<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { carouselContext } from "./carousel-context.ts";
import type { CarouselButtonExpose, CarouselSlotState } from "./carousel-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name for icon-only controls.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before navigation. Call `preventDefault()` to keep the active slide. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Control content. Receives the carousel state. */
  default(props: CarouselSlotState): unknown;
}>();

const context = carouselContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => !context.canScrollPrev.value);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.step(-1, "previous");
}

type CarouselPreviousSetupExpose = Omit<CarouselButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies CarouselPreviousSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    :aria-controls="context.viewportId.value"
    data-vize-ui="carousel-previous"
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
