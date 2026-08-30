<script setup lang="ts">
import { onMounted, onUpdated, toRef, useTemplateRef, watch, watchEffect } from "vue";

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
  size = false,
  safeArea = false,
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
   * Constrain the host to the available space with `max-width`/`max-height`
   * and publish `--vize-ui-positioner-available-width` and
   * `--vize-ui-positioner-available-height` custom properties.
   *
   * @default false
   */
  readonly size?: boolean;

  /**
   * Inset the active viewport by `env(safe-area-inset-*)` before collision
   * handling, keeping floating content clear of notches and rounded corners.
   *
   * @default false
   */
  readonly safeArea?: boolean;

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
  safeArea: toRef(() => safeArea),
  shift: toRef(() => shift),
  size: toRef(() => size),
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

// Placement styles are applied imperatively: the host geometry is
// measurement-driven output, not consumer-owned styling, and the authoring
// gate keeps `:style` bindings out of templates.
watchEffect(
  () => {
    if (element.value) {
      element.value.style.cssText = positioner.style.value;
    }
  },
  // Sync flush publishes the pre-measure fixed host as soon as the element
  // ref lands, so consumers never observe an unpositioned frame.
  { flush: "sync" },
);

defineExpose({
  arrowStyle: positioner.arrowStyle,
  availableHeight: positioner.availableHeight,
  availableWidth: positioner.availableWidth,
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
  >
    <slot :placement="positioner.resolvedPlacement.value" :ready="positioner.ready.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
