<script setup lang="ts">
import { onScopeDispose, useTemplateRef, watch } from "vue";

import { pagerContext } from "./pager-context.ts";

const { settleDelay = 120 } = defineProps<{
  /**
   * Quiet period (ms) after the last scroll event before a user scroll settles,
   * for engines without `scrollend`.
   *
   * @default 120
   */
  readonly settleDelay?: number;
}>();

defineSlots<{
  /** PagerPage panels, in page order. */
  default(props: Record<string, never>): unknown;
}>();

const context = pagerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
let timer: ReturnType<typeof setTimeout> | undefined;

function clearTimer(): void {
  if (timer !== undefined) clearTimeout(timer);
  timer = undefined;
}

function onScroll(): void {
  clearTimer();
  timer = setTimeout(() => {
    timer = undefined;
    context.settleScroll();
  }, settleDelay);
}

function onScrollEnd(): void {
  clearTimer();
  context.settleScroll();
}

watch(
  element,
  (target, _previous, onCleanup) => {
    context.registerViewport(target);
    if (target === null) return;
    target.addEventListener("scroll", onScroll, { passive: true });
    target.addEventListener("scrollend", onScrollEnd, { passive: true });
    onCleanup(() => {
      target.removeEventListener("scroll", onScroll);
      target.removeEventListener("scrollend", onScrollEnd);
    });
  },
  { flush: "post", immediate: true },
);

onScopeDispose(() => {
  clearTimer();
  context.registerViewport(null);
});
</script>

<template>
  <div ref="element" part="viewport" data-vize-ui="pager-viewport">
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Consumer CSS: display: flex; overflow-x: auto; scroll-snap-type: x mandatory. */
</style>
