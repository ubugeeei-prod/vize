<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { carouselContext } from "./carousel-context.ts";
import type {
  CarouselSlideExpose,
  CarouselSlideSlotState,
  CarouselSlideState,
} from "./carousel-types.ts";

const { index, ariaLabel = undefined } = defineProps<{
  /** Zero-based slide position. Must be unique and below `slideCount`. @default required */
  readonly index: number;

  /**
   * Accessible slide name. Defaults to "`index + 1` of `slideCount`" per the carousel pattern.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Slide content. Receives activity and visibility state. */
  default(props: CarouselSlideSlotState): unknown;
}>();

const context = carouselContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const slideId = computed(() => context.getSlideId(index));
const active = computed(() => context.index.value === index);
const inView = computed(() => context.isInView(index));
const state = computed<CarouselSlideState>(() => (active.value ? "active" : "inactive"));
const label = computed(
  () =>
    ariaLabel ??
    context.messages.value.slideLabel?.(index + 1, context.slideCount.value) ??
    `${index + 1} of ${context.slideCount.value}`,
);
const roleDescription = computed(() => context.messages.value.slide ?? "slide");
// Slides measured out of view leave the tab order and accessibility tree; the
// server and the first client render keep every slide interactive.
const inert = computed(() => inView.value === false && !active.value);
const slotState = computed<CarouselSlideSlotState>(() => ({
  active: active.value,
  inView: inView.value,
  index,
  state: state.value,
}));
let unregister: (() => void) | null = null;

function register(): void {
  unregister?.();
  unregister = element.value === null ? null : context.registerSlide(index, element.value);
}

onMounted(register);
watch(() => index, register, { flush: "post" });
onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

type CarouselSlideSetupExpose = Omit<
  CarouselSlideExpose,
  keyof CarouselSlideSlotState | "element" | "id"
> & {
  readonly active: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly inView: ComputedRef<boolean | null>;
  readonly index: number;
  readonly state: ComputedRef<CarouselSlideState>;
};

const exposed = {
  active,
  element,
  id: slideId,
  inView,
  index,
  state,
} satisfies CarouselSlideSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="slideId"
    ref="element"
    role="group"
    :aria-roledescription="roleDescription"
    :aria-label="label"
    :inert="inert ? true : undefined"
    data-vize-ui="carousel-slide"
    part="slide"
    :data-state="state"
    :data-index="index"
    :data-in-view="inView === null ? undefined : inView ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Consumers own slide sizing and snap alignment, for example:
   flex: 0 0 100%; scroll-snap-align: start; */
</style>
