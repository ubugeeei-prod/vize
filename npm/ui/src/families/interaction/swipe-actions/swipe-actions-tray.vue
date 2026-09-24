<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { swipeActionsContext } from "./swipe-actions-context.ts";
import type { SwipeActionsSide } from "./swipe-actions-types.ts";

const { side } = defineProps<{
  /**
   * Edge this tray is revealed from.
   *
   * @default required
   */
  readonly side: SwipeActionsSide;
}>();

defineSlots<{
  /** SwipeActionsAction buttons (or any focusable controls). */
  default(props: { readonly open: boolean }): unknown;
}>();

const context = swipeActionsContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const open = computed(() => context.open.value === side);

watch(element, (target) => context.registerTray(side, target), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerTray(side, null));
</script>

<template>
  <div
    ref="element"
    role="group"
    :inert="!open"
    part="tray"
    data-vize-ui="swipe-actions-tray"
    :data-side="side"
    :data-state="open ? 'open' : 'closed'"
  >
    <slot :open="open" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
