<script setup lang="ts">
import { computed } from "vue";

import { pagerContext } from "./pager-context.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name of the tab list.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** PagerTab segments, in page order. */
  default(props: Record<string, never>): unknown;
}>();

const context = pagerContext.use();

function onKeydown(event: KeyboardEvent): void {
  const pages = context.pages.value;
  const current = pages.indexOf(context.active.value);
  const rtl =
    event.currentTarget instanceof HTMLElement &&
    getComputedStyle(event.currentTarget).direction === "rtl";
  const forward = rtl ? "ArrowLeft" : "ArrowRight";
  const backward = rtl ? "ArrowRight" : "ArrowLeft";
  let target: string | undefined;
  if (event.key === forward) target = pages[(current + 1) % pages.length];
  else if (event.key === backward) target = pages[(current - 1 + pages.length) % pages.length];
  else if (event.key === "Home") target = pages[0];
  else if (event.key === "End") target = pages.at(-1);
  if (target === undefined) return;
  event.preventDefault();
  // Automatic activation: moving focus selects the page (APG tabs with automatic activation).
  context.select(target, "keyboard");
  context.focusTab(target);
}

// Composite widget: the tablist owns arrow-key navigation for its tabs.
const listProps = computed(() => ({
  role: "tablist",
  "aria-orientation": "horizontal" as const,
  "aria-label": ariaLabel,
  onKeydown,
}));
</script>

<template>
  <div v-bind="listProps" part="tab-list" data-vize-ui="pager-tab-list">
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Style as a segmented control. */
</style>
