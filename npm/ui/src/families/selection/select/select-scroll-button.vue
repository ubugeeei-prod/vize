<script setup lang="ts">
import { computed } from "vue";

import { selectContext } from "./select-context.ts";
import { useSelectScrollButton } from "./select-scroll.ts";
import type { SelectScrollDirection } from "./select-scroll.ts";

const {
  direction,
  step = 32,
  interval = 50,
} = defineProps<{
  /** Scroll direction driven while hovered or pressed. @default required */
  readonly direction: SelectScrollDirection;

  /**
   * Pixels scrolled per tick.
   *
   * @default 32
   */
  readonly step?: number;

  /**
   * Milliseconds between ticks while hovered or pressed.
   *
   * @default 50
   */
  readonly interval?: number;
}>();

defineSlots<{
  /** Decorative arrow content. Receives the scroll direction. */
  default(props: { readonly direction: SelectScrollDirection }): unknown;
}>();

const context = selectContext.use();
const scroll = useSelectScrollButton({
  direction: () => direction,
  interval: () => interval,
  step: () => step,
  viewport: () => context.viewportElement.value ?? context.contentElement.value,
});
const handlers = computed(() => ({
  onPointerdown: (event: PointerEvent) => {
    event.preventDefault();
    scroll.start();
  },
  onPointerenter: () => scroll.start(),
  onPointerleave: () => scroll.stop(),
  onPointerup: () => scroll.stop(),
}));

defineExpose({ scrollOnce: scroll.scrollOnce, update: scroll.update, visible: scroll.visible });
</script>

<template>
  <div
    v-bind="handlers"
    :hidden="scroll.visible.value ? undefined : true"
    aria-hidden="true"
    :data-vize-ui="`${context.partPrefix}-scroll-button`"
    part="scroll-button"
    :data-direction="direction"
  >
    <slot v-if="scroll.visible.value" :direction="direction" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
