<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { carouselContext } from "./carousel-context.ts";
import type { CarouselButtonExpose, CarouselIndicatorSlotState } from "./carousel-types.ts";

const { index, ariaLabel = undefined } = defineProps<{
  /** Zero-based slide index this indicator activates. @default required */
  readonly index: number;

  /**
   * Accessible name. Defaults to "Slide `index + 1`" when the slot renders no text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before activation. Call `preventDefault()` to keep the active slide. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Indicator content, e.g. a dot or thumbnail. Receives its activity state. */
  default(props: CarouselIndicatorSlotState): unknown;
}>();

const context = carouselContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const active = computed(() => context.index.value === index);
const disabled = computed(() => index < 0 || index >= context.slideCount.value);
const label = computed(() => ariaLabel ?? `Slide ${index + 1}`);
const slotState = computed<CarouselIndicatorSlotState>(() => ({ active: active.value, index }));
let unregister: (() => void) | null = null;

function register(): void {
  unregister?.();
  unregister = element.value === null ? null : context.registerIndicator(index, element.value);
}

onMounted(register);
watch(() => index, register, { flush: "post" });
onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.goTo(index, "indicator");
}

function onKeydown(event: KeyboardEvent): void {
  const vertical = context.orientation.value === "vertical";
  const rtl = context.dir.value === "rtl";
  const last = context.slideCount.value - 1;
  let target: number;
  if (event.key === (vertical ? "ArrowDown" : rtl ? "ArrowLeft" : "ArrowRight")) {
    target = index >= last ? (context.loop.value ? 0 : last) : index + 1;
  } else if (event.key === (vertical ? "ArrowUp" : rtl ? "ArrowRight" : "ArrowLeft")) {
    target = index <= 0 ? (context.loop.value ? last : 0) : index - 1;
  } else if (event.key === "Home") {
    target = 0;
  } else if (event.key === "End") {
    target = last;
  } else {
    return;
  }
  event.preventDefault();
  context.goTo(target, "indicator");
  void nextTick(() => context.focusIndicator(target));
}

type CarouselIndicatorSetupExpose = Omit<CarouselButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies CarouselIndicatorSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    role="tab"
    :disabled
    :tabindex="active ? 0 : -1"
    :aria-label="label"
    :aria-selected="active ? 'true' : 'false'"
    :aria-controls="context.getSlideId(index)"
    data-vize-ui="carousel-indicator"
    part="indicator"
    :data-state="active ? 'active' : 'inactive'"
    :data-index="index"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
