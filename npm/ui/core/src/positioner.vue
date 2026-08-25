<script setup lang="ts">
import { onMounted, onUpdated, toRef, useTemplateRef, watch } from "vue";

import { positionerContext } from "./positioner-context.ts";
import { usePositioner } from "./positioner-runtime.ts";
import type { Placement, PositionerElement, PositionerStrategy, Rect } from "./positioner-types.ts";

const {
  reference = null,
  placement = "bottom",
  strategy = "fixed",
  offset = 0,
  collisionPadding = 0,
  arrowPadding = 0,
  direction = "ltr",
  flip = true,
  shift = true,
  hide = true,
  updateOnScroll = true,
  updateOnResize = true,
  viewport = undefined,
} = defineProps<{
  /**
   * Reference element or virtual box the floating content is placed against.
   *
   * @default null
   */
  readonly reference?: PositionerElement | null;

  /**
   * Preferred placement before collision handling.
   *
   * @default "bottom"
   */
  readonly placement?: Placement;

  /**
   * CSS positioning mode published on the floating host.
   *
   * @default "fixed"
   */
  readonly strategy?: PositionerStrategy;

  /**
   * Gap on the main axis between reference and floating.
   *
   * @default 0
   */
  readonly offset?: number;

  /**
   * Viewport padding the floating element should not cross.
   *
   * @default 0
   */
  readonly collisionPadding?: number;

  /**
   * Inset kept between the arrow and floating edges.
   *
   * @default 0
   */
  readonly arrowPadding?: number;

  /**
   * Writing direction used to resolve start/end alignment.
   *
   * @default "ltr"
   */
  readonly direction?: "ltr" | "rtl";

  /**
   * Flip to the opposite side when the preferred side overflows more.
   *
   * @default true
   */
  readonly flip?: boolean;

  /**
   * Shift the floating box back into the viewport after flip.
   *
   * @default true
   */
  readonly shift?: boolean;

  /**
   * Hide when the reference no longer intersects the viewport.
   *
   * @default true
   */
  readonly hide?: boolean;

  /**
   * Recalculate while ancestors scroll.
   *
   * @default true
   */
  readonly updateOnScroll?: boolean;

  /**
   * Recalculate when the document or visual viewport resizes.
   *
   * @default true
   */
  readonly updateOnResize?: boolean;

  /**
   * Viewport used for flip, shift, and hide. Defaults to the visual viewport.
   *
   * @default undefined
   */
  readonly viewport?: Rect;
}>();

defineSlots<{
  /** Floating contents. Receives resolved placement for styling hooks. */
  default(props: { readonly placement: Placement; readonly ready: boolean }): unknown;
}>();

const positioner = usePositioner({
  arrowPadding: toRef(() => arrowPadding),
  collisionPadding: toRef(() => collisionPadding),
  direction: toRef(() => direction),
  flip: toRef(() => flip),
  hide: toRef(() => hide),
  offset: toRef(() => offset),
  placement: toRef(() => placement),
  shift: toRef(() => shift),
  strategy: toRef(() => strategy),
  updateOnResize: toRef(() => updateOnResize),
  updateOnScroll: toRef(() => updateOnScroll),
  viewport: toRef(() => viewport),
});
const element = useTemplateRef<HTMLDivElement>("element");

positionerContext.provide(positioner);

onMounted(() => {
  positioner.setFloating(element.value);
  positioner.setReference(reference);
});

watch(
  () => reference,
  (value) => {
    positioner.setReference(value);
  },
);

onUpdated(() => {
  positioner.setFloating(element.value);
});

defineExpose({
  arrowStyle: positioner.arrowStyle,
  element,
  hidden: positioner.hidden,
  ready: positioner.ready,
  resolvedPlacement: positioner.resolvedPlacement,
  style: positioner.style,
  update: positioner.update,
  x: positioner.x,
  y: positioner.y,
});
</script>

<template>
  <div
    ref="element"
    data-vize-ui="positioner"
    :data-vize-placement="positioner.resolvedPlacement.value"
    :data-vize-positioner-ready="positioner.ready.value ? 'true' : 'false'"
    :data-vize-hidden="positioner.hidden.value ? 'true' : undefined"
    :style="positioner.style.value"
  >
    <slot :placement="positioner.resolvedPlacement.value" :ready="positioner.ready.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
